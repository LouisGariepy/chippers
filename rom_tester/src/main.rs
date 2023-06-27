use chippers_core::interpreter::Interpreter;

fn main() {
    let program = include_bytes!("../flags.ch8");

    let mut interpreter = Interpreter::new(program);

    loop {
        print!("{esc}c", esc = 27 as char);
        interpreter.step();
        println!("{}", interpreter.screen);
        std::thread::sleep(std::time::Duration::from_nanos(1428571))
    }
}
