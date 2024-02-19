mod bin;
mod integrated;

pub type ProcClass<const N: usize> = ([u8; N], String, String);
pub struct CompiledClass<const N: usize> {
    cls: ProcClass<N>,
}

impl <const N : usize> CompiledClass<N> {
    pub fn name(&self) -> &String {
        &self.cls.1
    }

    pub fn class_content(&self) -> &String {
        &self.cls.2
    }

    pub fn bytes(&self) -> &[u8; N] {
        &self.cls.0
    }
}

impl<const N: usize> From<ProcClass<N>> for CompiledClass<N> {
    fn from(value: ProcClass<N>) -> Self {
        Self { cls: value }
    }
}

pub fn join<const N : usize>(lines: [&str; N]) -> String {
    // Add newline onto the end, as that's what we print
    format!("{}\n", lines.join("\n"))
}

pub use integrated::*;
pub use bin::*;
