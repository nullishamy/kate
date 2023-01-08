use unwind::{get_context, Cursor, RegNum};

pub fn unwind_stack() {
    get_context!(context);
    let mut cursor = Cursor::local(context).unwrap();

    loop {
        let ip = cursor.register(RegNum::IP).unwrap();

        match (cursor.procedure_info(), cursor.procedure_name()) {
            (Ok(ref info), Ok(ref name)) if ip == info.start_ip() + name.offset() => {
                println!(
                    "{:#016x} - {} ({:#016x}) + {:#x}",
                    ip,
                    name.name(),
                    info.start_ip(),
                    name.offset()
                );
            }
            _ => println!("{:#016x} - ????", ip),
        }

        if !cursor.step().unwrap() {
            break;
        }
    }
}
