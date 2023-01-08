#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IntegralType {
  Byte,
  Short, // AKA Char
  Int,
  Long
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FloatingType {
  Double,
  Float
}

#[derive(Debug, Clone)]
pub struct Integral {
  pub value: i64,
  pub ty: IntegralType
}

#[derive(Debug, Clone)]
pub struct Floating {
  pub value: f64,
  pub ty: FloatingType
}

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
  i8 => IntegralType::Byte,
  i16 => IntegralType::Short,
  i32 => IntegralType::Int,
  i64 => IntegralType::Long
);

from_num!(Floating > f64,
  f64 => FloatingType::Double,
  f32 => FloatingType::Float
);
