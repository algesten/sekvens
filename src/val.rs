use core::ops::{Add, Deref};

#[derive(Clone, Copy)]
pub struct Val<const S: i8, const T: i8>(pub i8);

impl<const S: i8, const T: i8> Val<S, T> {
    pub fn add(&mut self, v: i8) {
        let n = self.0.saturating_add(v).clamp(S, T);
        self.0 = n;
    }
}

impl<const S1: i8, const T1: i8, const S2: i8, const T2: i8> Add<Val<S2, T2>> for Val<S1, T1> {
    type Output = Val<S1, T1>;

    fn add(self, rhs: Val<S2, T2>) -> Self::Output {
        let mut o = self;
        Val::add(&mut o, rhs.0); // will clamp
        o
    }
}

impl<const S: i8, const T: i8> Deref for Val<S, T> {
    type Target = i8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
