use criterion::{black_box, criterion_group, criterion_main, Criterion};
extern crate rs8080_emulator;
use rs8080_emulator::{RS8080, DataBus};


struct DummyIO {}
impl DataBus for DummyIO {
    fn port_in(&mut self, x: u8) -> u8 {
        x
    }
    fn port_out(&mut self, _: u8, _: u8) {
    }
    fn port(&mut self, _: usize) -> &mut u8 {
        todo!()
    }
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut emu = RS8080::new(DummyIO {});
    let h = include_bytes!("../../roms/invaders.h");
    let g = include_bytes!("../../roms/invaders.g");
    let f = include_bytes!("../../roms/invaders.f");
    let e = include_bytes!("../../roms/invaders.e");
    emu.load_to_mem(h, 0);
    emu.load_to_mem(g, 0x0800);
    emu.load_to_mem(f, 0x1000);
    emu.load_to_mem(e, 0x1800);
    c.bench_function("emulate_next 20", |b| b.iter(|| emu.emulate_next()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);