use argparse::{ArgumentParser, Store, StoreTrue};
use emulator::State;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::fs;

mod emulator;
mod parser;

fn disassemble(input: &[u8], clean: bool) {
    let instructions = parser::disassemble(input);

    for line in instructions {
        if clean {
            println!("{}", line.instruction)
        } else {
            println!("{:#06x}: {:#06x}   {}", line.pos, line.op, line.instruction);
        }
    }
}

fn debugger(computer: &mut State) {
    let mut rl = Editor::<()>::new();
    let mut stepping = true;
    let mut breakpoints: Vec<usize> = Vec::new();

    while !computer.finished {
        if breakpoints.contains(&computer.pc()) {
            eprintln!("Hit breakpoint");
            stepping = true;
        }
        if stepping {
            println!("{}", computer.display());
            let readline = rl.readline("→ ");
            match readline {
                Ok(cmd) => {
                    rl.add_history_entry(cmd.as_str());
                    if cmd.starts_with('c') {
                        stepping = false;
                        computer.execute();
                    } else if cmd.starts_with('m') {
                        let tokens = cmd.split_whitespace().collect::<Vec<&str>>();
                        let from = usize::from_str_radix(tokens[1], 16).unwrap();
                        let to = usize::from_str_radix(tokens[2], 16).unwrap();
                        computer.dump_memory(from, to);
                    } else if cmd.starts_with('b') {
                        let tokens = cmd.split_whitespace().collect::<Vec<&str>>();
                        if tokens.len() == 2 {
                            match usize::from_str_radix(tokens[1], 16) {
                                Ok(num) => breakpoints.push(num),
                                Err(_) => eprintln!("Could not parse address"),
                            };
                        } else {
                            eprintln!("Breakpoints:");
                        }
                    } else {
                        computer.execute();
                    }
                }
                Err(ReadlineError::Interrupted) => {
                    println!("Abort");
                    break;
                }
                Err(ReadlineError::Eof) => {
                    println!("EOF");
                    break;
                }
                Err(err) => {
                    println!("Error: {:?}", err);
                    break;
                }
            }
        } else {
            computer.execute();
        }
    }
}

fn emulate(input: &[u8], stdin: Vec<u8>, debug: bool) {
    let mut memory: [u8; 4096] = [0; 4096];
    let mut l = input.len();
    if l > 4096 {
        l = 4096;
    }
    for (i, v) in input[7..l].iter().enumerate() {
        memory[i] = *v;
    }
    let mut computer = State::new(memory, stdin);

    if debug {
        debugger(&mut computer);
    } else {
        while !computer.finished {
            computer.execute()
        }
    }
    eprintln!("End state:\n{}", computer.display());
    println!("{}", hex::encode(computer.stdout()));
}

fn main() {
    let mut file = String::new();
    let mut clean: bool = false;
    let mut disass: bool = false;
    let mut debug: bool = false;
    let mut stdin = String::new();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Disassemble s8");
        ap.refer(&mut file)
            .add_argument("file", Store, "Filename to load")
            .required();
        ap.refer(&mut stdin)
            .add_argument("stdin", Store, "Input data")
            .required();
        ap.refer(&mut clean)
            .add_option(&["-c", "--clean"], StoreTrue, "Output just s8asm");
        ap.refer(&mut disass).add_option(
            &["-D", "--disassemble"],
            StoreTrue,
            "Do disassembly instead of running",
        );
        ap.refer(&mut debug)
            .add_option(&["-d", "--debug"], StoreTrue, "Run in debug mode");

        ap.parse_args_or_exit();
    }

    let input = fs::read(file).expect("Could not read input file");
    let stdin = fs::read(stdin).expect("Second argument is a file with indata.");

    if disass {
        disassemble(&input, clean);
    } else {
        emulate(&input, stdin, debug);
    }

    std::process::exit(0);
}
