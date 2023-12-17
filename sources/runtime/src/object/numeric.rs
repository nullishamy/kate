#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntegralType {
    Int,
    Long,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FloatingType {
    Double,
    Float,
}

#[derive(Debug, Clone, Copy)]
pub struct Integral {
    pub value: i64,
    pub ty: IntegralType,
}

#[derive(Debug, Clone, Copy)]
pub struct Floating {
    pub value: f64,
    pub ty: FloatingType,
}

pub const TRUE: Integral = Integral {
    value: 1,
    ty: IntegralType::Int,
};

pub const FALSE: Integral = Integral {
    value: 0,
    ty: IntegralType::Int,
};

macro_rules! from_num {
    ($b: ty > $a: ty, $( $x:ty => $y: expr ),* ) => {
        $(
            impl From<$x> for $b {
              fn from(value: $x) -> Self {
                Self {
                  value: value as $a,
                  ty: $y
                }
              }
            }
        )*
    };
}

from_num!(Integral > i64,
  i32 => IntegralType::Int,
  i64 => IntegralType::Long
);

from_num!(Floating > f64,
  f64 => FloatingType::Double,
  f32 => FloatingType::Float
);
