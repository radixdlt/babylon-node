use crate::prelude::*;
use std::any::type_name;

pub trait PanickingOps: Sized {
    fn add_or_panic(self, rhs: Self) -> Self;
    fn sub_or_panic(self, rhs: Self) -> Self;
    fn mul_or_panic(self, rhs: Self) -> Self;
    fn div_or_panic(self, rhs: Self) -> Self;
    fn neg_or_panic(self) -> Self;
}

impl<T> PanickingOps for T
where
    T: CheckedAdd<Output = T>
        + CheckedSub<Output = T>
        + CheckedDiv<Output = T>
        + CheckedMul<Output = T>
        + CheckedNeg<Output = T>
        + Copy
        + Display,
{
    fn add_or_panic(self, rhs: Self) -> Self {
        op_or_panic(self, "+", rhs, self.checked_add(rhs))
    }

    fn sub_or_panic(self, rhs: Self) -> Self {
        op_or_panic(self, "-", rhs, self.checked_sub(rhs))
    }

    fn mul_or_panic(self, rhs: Self) -> Self {
        op_or_panic(self, "*", rhs, self.checked_mul(rhs))
    }

    fn div_or_panic(self, rhs: Self) -> Self {
        op_or_panic(self, "/", rhs, self.checked_div(rhs))
    }

    fn neg_or_panic(self) -> Self {
        if let Some(result) = self.checked_neg() {
            result
        } else {
            panic!("result of -{} does not fit in {}", self, type_name::<T>());
        }
    }
}

pub trait PanickingSumIterator<E> {
    fn sum_or_panic(self) -> E;
}

impl<T, E> PanickingSumIterator<E> for T
where
    T: Iterator<Item = E>,
    E: Default + CheckedAdd<Output = E> + Copy + Display,
{
    fn sum_or_panic(self) -> E {
        let mut result = E::default();
        for (index, element) in self.enumerate() {
            let sum = result.checked_add(element);
            if let Some(sum) = sum {
                result = sum;
            } else {
                panic!(
                    "result of accumulating {}. element ({} + {}) does not fit in {}",
                    index,
                    result,
                    element,
                    type_name::<T>()
                );
            }
        }
        result
    }
}

fn op_or_panic<T: Display>(left: T, op: &str, right: T, result: Option<T>) -> T {
    if let Some(result) = result {
        result
    } else {
        panic!(
            "result of {} {} {} does not fit in {}",
            left,
            op,
            right,
            type_name::<T>()
        );
    }
}
