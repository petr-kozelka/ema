/// Exponential Moving Average
///
/// https://www.investopedia.com/terms/e/ema.asp

const N_OBSERVATIONS: usize = 5 /*30*/; // NOTE I use smaller number just to keep the test data small, and also to make the impact of history higher
const SMOOTHING: f64 = 2.0;
const ALPHA: f64 = SMOOTHING / (1 + N_OBSERVATIONS) as f64;


pub trait EmaComputation {
    fn get_result(&self) -> Option<f64>;
    fn update(&mut self, new_value: f64);
}

fn compute_ema(prev_ema: f64, price: f64) -> f64 {
    prev_ema + ALPHA * (price - prev_ema)
}

#[derive(Default)]
pub struct EmaFast {
    result: Option<f64>,
    last: f64,
    count: usize,
}

impl EmaComputation for EmaFast {
    fn get_result(&self) -> Option<f64> {
        self.result
    }

    /// This algorithm is fast, because it executes the [`compute_ema`] function only once.
    /// However, it does not isolate the result from the effect of values outside the window.
    fn update(&mut self, new_price: f64) {
        if self.count == 0 {
            self.last = new_price;
        } else {
            self.last = compute_ema(self.last, new_price)
        }
        self.count += 1;
        self.result = if self.count < N_OBSERVATIONS {
            None
        } else {
            Some(self.last)
        }
    }
}


#[derive(Default)]
pub struct EmaCorrect {
    result: Option<f64>,
    /// first value is oldest, last is newest
    window: Vec<f64>,
}

impl EmaComputation for EmaCorrect {
    fn get_result(&self) -> Option<f64> {
        self.result
    }

    /// This algorithm performs fresh computation on the entire window, and is therefore much slower.
    /// But it matches the logic of "window of observations" as implied by the semantics of _moving average_.
    fn update(&mut self, new_price: f64) {
        if self.window.len() == N_OBSERVATIONS {
            self.window.remove(0);
        }
        self.window.push(new_price);
        if self.window.len() < N_OBSERVATIONS {
            self.result = None;
        } else {
            let mut result = self.window[0];
            for i in 1..N_OBSERVATIONS {
                result = compute_ema(result, self.window[i]);
            }
            self.result = Some(result);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{EmaComputation, EmaCorrect, EmaFast, N_OBSERVATIONS};

    /// This is reused by both implementation for testing the first provided value, which is the same in both algos.
    /// Second and further values are different, and so they will be tested separately.
    fn basic_tests<E: EmaComputation>(ema: &mut E) {
        for i in 1..N_OBSERVATIONS {
            ema.update(i as f64);
            assert_eq!(None, ema.get_result(), "'None' expected at position {i}");
        }
        ema.update(N_OBSERVATIONS as f64);
        assert_eq!(Some(3.3950617283950617), ema.get_result(), "first provided value");
    }

    #[test]
    fn fast_basic_tests() {
        let mut ema_fast = EmaFast::default();
        basic_tests(&mut ema_fast);
        ema_fast.update(N_OBSERVATIONS as f64);
        // here is the difference: following value (in first arg) is IMHO incorrect
        assert_eq!(Some(3.9300411522633745), ema_fast.get_result(), "second provided value");
    }

    #[test]
    fn correct_basic_tests() {
        let mut ema_correct = EmaCorrect::default();
        basic_tests(&mut ema_correct);
        ema_correct.update(N_OBSERVATIONS as f64);
        // here is the difference:
        assert_eq!(Some(4.061728395061729), ema_correct.get_result(), "second provided value");
    }

    #[test]
    fn compare_fast_and_correct() {
        // fast variant
        let secondval_fast = {
            let mut ema_fast = EmaFast::default();
            basic_tests(&mut ema_fast);
            ema_fast.update(N_OBSERVATIONS as f64);
            ema_fast.get_result()
        };
        // correct variant
        let secondval_correct = {
            let mut ema_correct = EmaCorrect::default();
            basic_tests(&mut ema_correct);
            ema_correct.update(N_OBSERVATIONS as f64);
            ema_correct.get_result()
        };

        // the proof
        assert_eq!(secondval_fast, secondval_correct, "this failure proves that the computation differs between the two algos");
    }
}
