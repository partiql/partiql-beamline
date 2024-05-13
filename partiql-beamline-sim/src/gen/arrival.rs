use crate::primitives::Tick;
use rand::Rng;
use statrs::distribution::Exp;
use std::cell::RefCell;
use std::fmt::{Debug, Formatter};
use std::ops::DerefMut;
use time::Duration;

use crate::gen::ArrivalTime;
use rand_distr::Distribution;

/// A stochastic process arrival time generator for homogenenously distributed arrivals.
///
/// see https://en.wikipedia.org/wiki/Poisson_point_process#Homogeneous_Poisson_point_process
#[derive(Clone)]
pub struct HomogeneousPoisson<R>
where
    R: Rng + Sized + Clone,
{
    /// The mean time between arrivals
    interarrival_time: Duration,

    /// An exponential distribution used to generate the next interarrival time
    exp: Exp,

    /// The source of randomness
    rng: RefCell<R>,
}

impl<R> Debug for HomogeneousPoisson<R>
where
    R: Rng + Sized + Clone,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HomogeneousPoisson")
            .field("interarrival_time", &self.interarrival_time)
            .finish()
    }
}

impl<R> HomogeneousPoisson<R>
where
    R: Rng + Sized + Clone,
{
    pub fn from_interarrival_time(rng: R, interarrival_time: Duration) -> Self {
        let rate = interarrival_time.as_seconds_f64();
        let exp = Exp::new(1f64 / rate).expect("dist");
        let rng = RefCell::new(rng);

        HomogeneousPoisson {
            interarrival_time,
            exp,
            rng,
        }
    }
}

impl<R> ArrivalTime for HomogeneousPoisson<R>
where
    R: Rng + Sized + Clone,
{
    fn next_arrival(&self, now: Tick) -> Tick {
        let mut rng = self.rng.borrow_mut();
        let rng = rng.deref_mut();
        let next = self.exp.sample(rng);
        let millis = Duration::seconds_f64(next).whole_milliseconds() as u128;
        Tick(now.0 + millis)
    }
}
