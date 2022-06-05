use crate::runtime::bytecode::args::Args;
use crate::runtime::stack::{Stack, StackValue};
use crate::structs::descriptor::MethodDescriptor;
use anyhow::anyhow;

pub fn create_args(
    descriptor: &MethodDescriptor,
    ops: &mut Stack<StackValue>,
) -> anyhow::Result<Args> {
    let mut args = vec![];

    let mut idx = 0;
    let sig_args = &descriptor.parameters;

    while let Some(target) = sig_args.get(idx) {
        let from = ops.peek();

        // if we peek and it's None it means there's no more elements
        if from.is_none() {
            return Err(anyhow!(
                "insufficient args passed for method {:?}",
                descriptor
            ));
        }

        let from = from.unwrap();
        let from_t = from.get_type();
        let from_t = from_t.as_ref();

        // FIXME: logic error - if from_t is None, the type wasn't a reference type, but a primitive
        if from_t.is_none() {
            // null case
            // pop because its quicker & more accurate.
            // this could technically be a construction of RefType::Null
            args.push(
                ops.pop()
                    .expect("somehow we could not pop the value? we just peeked it though..."),
            )
        }

        let from_t = from_t.unwrap();

        if from_t == target {
            // pop the value off, we know it exists now and has the right type
            args.push(
                ops.pop()
                    .expect("somehow we could not pop the value? we just peeked it though..."),
            )
        } else {
            // invalid type
            return Err(anyhow!(
                "type mismatch for method {:?}, expected {:?} got {:?}",
                descriptor,
                target,
                from_t
            ));
        }

        idx += 1;
    }

    Ok(Args { entries: args })
}
