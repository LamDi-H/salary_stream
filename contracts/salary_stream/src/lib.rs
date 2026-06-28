#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

/// On-chain state for a single salary stream. The stream is parameterised
/// by a per-second rate and a total duration; the employee is paid for
/// every second the stream is in the `Active` and not `Paused` state.
#[contracttype]
#[derive(Clone)]
pub struct Stream {
    /// Account that opened the stream and is the source of funds.
    pub employer: Address,
    /// Account that is entitled to withdraw accrued salary.
    pub employee: Address,
    /// Accrual rate expressed in smallest salary units per second.
    pub rate_per_second: u64,
    /// Total number of seconds the stream is allowed to accrue salary.
    pub duration_seconds: u64,
    /// Ledger timestamp of the last time the accrual counters were updated.
    pub last_update: u64,
    /// Total number of seconds that have already been counted toward the
    /// stream's duration (always `<= duration_seconds`).
    pub accrued_seconds: u64,
    /// Total amount already withdrawn by the employee.
    pub withdrawn: u64,
    /// Whether the stream is currently paused by the employer.
    pub paused: bool,
    /// Whether the stream is still open. Becomes `false` after `close`.
    pub active: bool,
}

/// Storage key namespace. One `Stream` record is stored per `stream_id`.
#[contracttype]
pub enum DataKey {
    Stream(Symbol),
}

fn load_stream(env: &Env, stream_id: &Symbol) -> Stream {
    env.storage()
        .instance()
        .get(&DataKey::Stream(stream_id.clone()))
        .expect("stream not found")
}

fn save_stream(env: &Env, stream_id: &Symbol, stream: &Stream) {
    env.storage()
        .instance()
        .set(&DataKey::Stream(stream_id.clone()), stream);
}

/// Mutate `stream` so that time elapsed since `last_update` is folded
/// into `accrued_seconds`, respecting the `duration_seconds` cap and
/// skipping periods in which the stream was paused or closed.
fn accrue_to_now(stream: &mut Stream, now: u64) {
    if stream.active && !stream.paused {
        let period = now.saturating_sub(stream.last_update);
        let remaining = stream.duration_seconds.saturating_sub(stream.accrued_seconds);
        let add = period.min(remaining);
        stream.accrued_seconds = stream.accrued_seconds.saturating_add(add);
    }
    stream.last_update = now;
}

/// Pure projection of how much salary is currently unclaimed, without
/// mutating the stored stream record.
fn projected_unclaimed(stream: &Stream, now: u64) -> u64 {
    let mut accrued_seconds = stream.accrued_seconds;
    if stream.active && !stream.paused {
        let period = now.saturating_sub(stream.last_update);
        let remaining = stream.duration_seconds.saturating_sub(accrued_seconds);
        let add = period.min(remaining);
        accrued_seconds = accrued_seconds.saturating_add(add);
    }
    let total = accrued_seconds.saturating_mul(stream.rate_per_second);
    total.saturating_sub(stream.withdrawn)
}

#[contract]
pub struct SalaryStream;

#[contractimpl]
impl SalaryStream {
    /// Open a new salary stream from `employer` to `employee` at
    /// `rate_per_second` for a total of `duration_seconds`. Only the
    /// employer may open a stream, and the `stream_id` must be unique.
    pub fn open_stream(
        env: Env,
        employer: Address,
        employee: Address,
        stream_id: Symbol,
        rate_per_second: u32,
        duration_seconds: u32,
    ) {
        employer.require_auth();

        if rate_per_second == 0 || duration_seconds == 0 {
            panic!("rate and duration must be positive");
        }

        let key = DataKey::Stream(stream_id.clone());
        if env.storage().instance().has(&key) {
            panic!("stream already exists");
        }

        let now = env.ledger().timestamp();
        let stream = Stream {
            employer: employer.clone(),
            employee: employee.clone(),
            rate_per_second: rate_per_second as u64,
            duration_seconds: duration_seconds as u64,
            last_update: now,
            accrued_seconds: 0,
            withdrawn: 0,
            paused: false,
            active: true,
        };
        env.storage().instance().set(&key, &stream);
    }

    /// Employee withdraws all salary that has accrued up to the current
    /// ledger timestamp. Returns the amount withdrawn. Only the
    /// employee of the stream may call this.
    pub fn withdraw(env: Env, employee: Address, stream_id: Symbol) -> u64 {
        employee.require_auth();

        let mut stream: Stream = load_stream(&env, &stream_id);
        if stream.employee != employee {
            panic!("not the employee of this stream");
        }
        if !stream.active {
            panic!("stream is closed");
        }

        let now = env.ledger().timestamp();
        accrue_to_now(&mut stream, now);

        let total = stream.accrued_seconds.saturating_mul(stream.rate_per_second);
        let claimable = total.saturating_sub(stream.withdrawn);
        stream.withdrawn = stream.withdrawn.saturating_add(claimable);
        save_stream(&env, &stream_id, &stream);

        claimable
    }

    /// Employer pauses the stream. No salary accrues while paused; the
    /// employee may still withdraw salary that accrued before the pause.
    pub fn pause(env: Env, employer: Address, stream_id: Symbol) {
        let mut stream: Stream = load_stream(&env, &stream_id);
        if stream.employer != employer {
            panic!("not the employer of this stream");
        }
        if !stream.active {
            panic!("stream is closed");
        }
        if stream.paused {
            panic!("stream already paused");
        }

        employer.require_auth();

        let now = env.ledger().timestamp();
        accrue_to_now(&mut stream, now);
        stream.paused = true;
        save_stream(&env, &stream_id, &stream);
    }

    /// Employer resumes a previously paused stream. Salary begins
    /// accruing again from the current ledger time, with the
    /// `duration_seconds` cap still enforced.
    pub fn resume(env: Env, employer: Address, stream_id: Symbol) {
        let mut stream: Stream = load_stream(&env, &stream_id);
        if stream.employer != employer {
            panic!("not the employer of this stream");
        }
        if !stream.active {
            panic!("stream is closed");
        }
        if !stream.paused {
            panic!("stream is not paused");
        }

        employer.require_auth();

        // Reset the accrual clock so the paused interval is not
        // double-counted when more time elapses.
        stream.last_update = env.ledger().timestamp();
        stream.paused = false;
        save_stream(&env, &stream_id, &stream);
    }

    /// Employer closes the stream. Any salary already accrued remains
    /// claimable by the employee; after closing the stream cannot be
    /// re-opened or paused.
    pub fn close(env: Env, employer: Address, stream_id: Symbol) {
        let mut stream: Stream = load_stream(&env, &stream_id);
        if stream.employer != employer {
            panic!("not the employer of this stream");
        }
        if !stream.active {
            panic!("stream already closed");
        }

        employer.require_auth();

        let now = env.ledger().timestamp();
        accrue_to_now(&mut stream, now);
        stream.active = false;
        save_stream(&env, &stream_id, &stream);
    }

    /// View the currently unclaimed (accrued minus already withdrawn)
    /// salary for `stream_id`, projected to the current ledger
    /// timestamp. The result is encoded as a `u32`; the value is
    /// capped at `u32::MAX` for the return type.
    pub fn get_accrued(env: Env, stream_id: Symbol) -> u32 {
        let stream: Stream = load_stream(&env, &stream_id);
        let now = env.ledger().timestamp();
        let unclaimed = projected_unclaimed(&stream, now);
        unclaimed.min(u32::MAX as u64) as u32
    }
}
