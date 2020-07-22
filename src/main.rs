use std::fs::{File, OpenOptions};
use memmap::Mmap;
use std::collections::HashSet;
use std::env;
use std::io::Write;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 || args.len() > 4 || (args[1] != "decode" && args[1] != "encode") {
        println!("Usage: {} <decode | encode> <encode: region_file | decode: encoded_region_file> <encode: file_to_add | decode: output>", args[0]);
        return
    }
    if args[1] == "encode" {
        let file = OpenOptions::new().read(true).write(true).open(args[2].as_str()).expect("Failed to file");
        let to_encode = File::open(args[3].as_str()).expect("Failed to file");

        let mmap = unsafe { Mmap::map(&file).expect("Failed to mmap") };
        let inputmmap = unsafe { Mmap::map(&to_encode).expect("Failed to mmap") };

        let mut sector: i32 = 8192;

        let mut sectors = Vec::new();
        let mut sector_free_space = Vec::new();
        let mut started_sectors: HashSet<i32> = HashSet::new();

        loop {
            sectors.push(-1);
            sector += 4096;

            if sector > mmap.len() as i32 {
                break
            }
        }

        for i in 0..1024 {
            let offset = ((mmap[(i * 4) as usize] as i32) << 16) +
                ((mmap[(i * 4) as usize + 1] as i32) << 8) +
                (mmap[(i * 4) as usize + 2] as i32);
            let size = mmap[(i * 4) as usize + 3] as i32;
            for j in offset..offset + size {
                sectors[j as usize - 2] = i as i32;
            }
        }

        let mut sector = 8192;
        for i in &sectors {
            if *i == -1 {
                sector_free_space.push(4096);
            } else {
                if !started_sectors.contains(&i) {
                    // read sector header

                    started_sectors.insert(*i);

                    let mut size = (mmap[sector] as i32) << 24;
                    size += (mmap[sector + 1] as i32) << 16;
                    size += (mmap[sector + 2] as i32) << 8;
                    size += mmap[sector + 3] as i32;
                    size += 4;

                    let sectors_used = (size - 1) / 4096;

                    if sectors_used == 0 {
                        sector_free_space.push(4096 - size);
                    } else {
                        for _i in 0..sectors_used {
                            sector_free_space.push(0);
                        }

                        sector_free_space.push(4096 - (size % 4096));
                    }
                }
            }
            sector += 4096;
        }

        let mut index: usize = 0;
        let mut free_space = 0;
        for fs in 0..sector_free_space.len() {
            if sector_free_space[fs] < 20 {
                sector_free_space[index] = 0
            } else {
                free_space += sector_free_space[fs];
            }

            index += 1;
        }

        println!("{} bytes space was available in this file", free_space);

        if free_space < inputmmap.len() as i32 {
            panic!("Too big!");
        }

        let mut targetmap = mmap.make_mut().expect("Failed to make map writable");
        let mut ptr = 0;
        let mut broken = false;
        let mut sindex = 8192;
        let header = inputmmap.len().to_be_bytes();

        for fs in sector_free_space {
            let start = sindex + (4096 - fs) as usize;
            for i in 0..fs {
                if ptr < 8 {
                    targetmap[start + i as usize] = header[ptr];
                } else {
                    targetmap[start + i as usize] = inputmmap[ptr - 8];
                }
                ptr += 1;

                if ptr > (inputmmap.len() + 7) {
                    broken = true;
                    break;
                }
            }

            sindex += 4096;
            if broken { break };
        }
    }

    if args[1] == "decode" {
        let file = OpenOptions::new().read(true).open(args[2].as_str()).expect("Failed to file");
        let mmap = unsafe { Mmap::map(&file).expect("Failed to mmap") };

        let mut sector: i32 = 8192;

        let mut sectors = Vec::new();
        let mut sector_free_space = Vec::new();
        let mut started_sectors: HashSet<i32> = HashSet::new();

        loop {
            sectors.push(-1);
            sector += 4096;

            if sector > mmap.len() as i32 {
                break
            }
        }

        for i in 0..1024 {
            let offset = ((mmap[(i * 4) as usize] as i32) << 16) +
                ((mmap[(i * 4) as usize + 1] as i32) << 8) +
                (mmap[(i * 4) as usize + 2] as i32);
            let size = mmap[(i * 4) as usize + 3] as i32;
            for j in offset..offset + size {
                sectors[j as usize - 2] = i as i32;
            }
        }

        let mut sector = 8192;
        for i in &sectors {
            if *i == -1 {
                sector_free_space.push(4096);
            } else {
                if !started_sectors.contains(&i) {
                    // read sector header

                    started_sectors.insert(*i);

                    let mut size = (mmap[sector] as i32) << 24;
                    size += (mmap[sector + 1] as i32) << 16;
                    size += (mmap[sector + 2] as i32) << 8;
                    size += mmap[sector + 3] as i32;
                    size += 4;

                    let sectors_used = (size - 1) / 4096;

                    if sectors_used == 0 {
                        sector_free_space.push(4096 - size);
                    } else {
                        for _i in 0..sectors_used {
                            sector_free_space.push(0);
                        }

                        sector_free_space.push(4096 - (size % 4096));
                    }
                }
            }
            sector += 4096;
        }

        let mut index: usize = 0;
        for fs in 0..sector_free_space.len() {
            if sector_free_space[fs] < 20 {
                sector_free_space[index] = 0
            } else {

            }

            index += 1;
        }

        let mut extracted_data: Vec<u8> = Vec::new();
        let mut size : usize = 0;
        let mut ptr : usize = 0;

        let mut broken = false;
        let mut sindex = 8192;

        for fs in sector_free_space {
            let start = sindex + (4096 - fs) as usize;
            for i in 0..fs {
                if ptr < 8 {
                    size += ((mmap[start + i as usize] as u64) << ((7 - ptr) * 8)) as usize;
                } else {
                    extracted_data.push(mmap[start + i as usize])
                }
                ptr += 1;

                if ptr > 7 {
                    if ptr > (size + 7) {
                        broken = true;
                        break;
                    }
                }
            }

            sindex += 4096;
            if broken { break };
        }

        println!("{}", extracted_data.len());

        let mut buffer = File::create(args[3].as_str()).expect("failed to create");
        buffer.write_all(&*extracted_data).expect("Failed to write file");
    }
}
