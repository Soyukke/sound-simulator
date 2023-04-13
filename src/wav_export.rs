use wav::*;
use std::fs::File;
use std::path::Path;
use std::f64::consts::PI;

use crate::WaveChart;
pub trait WavExport {
    fn export(self) -> std::io::Result<()>;
}

impl WavExport for WaveChart {
    fn export(self) -> std::io::Result<()> {
        let h = Header::new(WAV_FORMAT_PCM, 1, 48_000, 16);
        let n_data = 48_000 * 5;
        let mut data: Vec<i16> = vec![0; n_data];
        let f = 440.;
        for i in 0..n_data {
            let x = f64::from(i as i32) * 2.0*PI*f/48_000.0;
            // 0 ~ 1.0
            let y = (f64::sin(x) + 1.0) / 2.0;
            let y = (y * f64::from(i16::MAX)) as i16;
            data[i] = y;
        }
        let track = BitDepth::Sixteen(data);
        let mut out_file = File::create(Path::new("data/output.wav"))?;
        wav::write(h, &track, &mut out_file)?;
        Ok(())
    }
}
