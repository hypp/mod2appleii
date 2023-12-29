use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::io::BufReader;
use std::io::Read;

extern crate modfile;
use modfile::ptmf;

#[macro_use]
extern crate serde_derive;
extern crate docopt;
use docopt::Docopt;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

static USAGE: &'static str = "
mod2appleii.

Usage: 
    mod2appleii (-h | --help)
    mod2appleii (-V | --version)
    mod2appleii --in=<filename> --out=<filename>

Options:
    -V, --version         Show version info.
    -h, --help            Show this text.
	--in=<filename>       Name of inputfile
	--out=<filename>      Name of outputfile
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_help: bool,
	flag_version: bool,
	
	flag_in: String,
	flag_out: String,
}

fn note_from_period(period: u16, octave_add: i32) -> String {
	// Find the position in PERIODS with the
	// smallest difference
	let mut found:i32 = -1;
	let mut min_diff = 65536;
	let key = period as i32;
	for i in 0..ptmf::PERIODS.len() {
		let diff = (key as i32 - ptmf::PERIODS[i] as i32).abs();
		if diff < min_diff {
			min_diff = diff;
			found = i as i32;
		}
	}
	
	let note = if found == -1 {
		println!("Failed to find note name");
		String::new()
	} else {
		let octave = found / 12 + octave_add;
		let name = ptmf::NOTE_NAMES[(found % 12) as usize];
        format!("{}{}",name.to_lowercase(),octave)
	};

	note
}

fn is_pattern_break(channels:&Vec<ptmf::Channel>) -> bool {
	// Check for pattern break
	let effect = channels[0].effect | channels[1].effect | channels[2].effect | channels[3].effect;
	if effect & 0x0f00 == 0x0d00 {
		true
	} else {
		false
	}
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());
//    println!("{:?}", args);	
	
	if args.flag_version {
		println!("Version: {}", VERSION);
		return;
	}
	
	if args.flag_in == "" {
		println!("No inputfile specificed");
		return
	}

	if args.flag_out == "" {
		println!("No outputfile specificed");
		return
	}

	let ref input_filename = args.flag_in;
	let file = match File::open(input_filename) {
		Ok(file) => file,
		Err(e) => {
			println!("Failed to open file: '{}' Error: '{}'", input_filename, e);
			return
		}
	};

	let read_fn:fn (&mut dyn Read, bool) -> Result<ptmf::PTModule, ptmf::PTMFError> = ptmf::read_mod;

	let mut reader = BufReader::new(&file);
	let module = match read_fn(&mut reader, true) {
		Ok(module) => module,
		Err(e) => {
			println!("Failed to parse file: '{}' Error: '{:?}'", input_filename, e);
			return;
		}
	};

	let ref output_filename = args.flag_out;
	let file = match File::create(&output_filename) {
		Ok(file) => file,
		Err(e) => {
			println!("Failed to open file: '{}' Error: '{:?}'", output_filename, e);
			return
		}
	};

	let mut writer = BufWriter::new(&file);		

    for i in 0..module.length {
        let position = module.positions.data[i as usize];
        let pattern = &module.patterns[position as usize];

        // each row in module is one 16th note
        let mut note = String::from("");
        let mut duration = 0;
        for row_idx in 0..pattern.rows.len() {
            let row = &pattern.rows[row_idx];
            // assume a row with only rests
            let mut new_note = String::from("R");
            for channel_idx in 0..row.channels.len() {
                let octave_add = if channel_idx > 0 {
                    1
                } else {
                    0
                };
                let channel = &row.channels[channel_idx];
				if channel.period > 0 {
					new_note = note_from_period(channel.period,octave_add);
				}
            }

            // First row is special
            if row_idx == 0 {
                note = new_note.clone();
                duration = 1;
            }
            else
            {
                if new_note == note {
                    // Same note as before, keep going
                    duration += 1;
                }
                else 
                {
                    write!(writer,"{} {} ", note, duration).unwrap();

                    note = new_note.clone();
                    duration = 1;
                }
            }

            if is_pattern_break(&row.channels) {
                break;
            }
        }

        writeln!(writer,"{} {} ", note, duration).unwrap();

    }


}

