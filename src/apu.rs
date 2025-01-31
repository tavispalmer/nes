use std::ptr;

use crate::nsf::{CNSFCore, CNSFFile};

pub struct Apu {
    // nsf core
    nsf: Box<CNSFCore>,
}

impl Apu {
    pub fn new() -> Self {
        const NSF_DUMMY_CODE: [u8; 3] = [0x4c,0x00,0x80];

        // create dummy file
        let mut nsf_dummy_file = CNSFFile::new();
        nsf_dummy_file.nLoadAddress = 0x8000;
        nsf_dummy_file.nInitAddress = 0x8000;
        nsf_dummy_file.nPlayAddress = 0x8000;

        nsf_dummy_file.nNTSC_PlaySpeed = 16666;
        nsf_dummy_file.nTrackCount = 1;

        nsf_dummy_file.pDataBuffer = NSF_DUMMY_CODE.as_ptr() as _;
        nsf_dummy_file.nDataBufferSize = NSF_DUMMY_CODE.len() as _;

        let mut nsf = Box::new(CNSFCore::new());
        nsf.Initialize();
        nsf.SetPlaybackOptions(48000, 2);
        nsf.LoadNSF(&nsf_dummy_file);
        nsf.SetTrack(0);
        nsf.SetChannelOptions(0,1, 255, -45, 1);
        nsf.SetChannelOptions(1, 1, 255, 45, 1);
        nsf.SetChannelOptions(2, 1, 255, 0, 0);
        nsf.SetChannelOptions(3, 1, 255, 0, 0);

        nsf_dummy_file.pDataBuffer = ptr::null_mut();

        Self {
            nsf
        }
    }

    pub fn tick(&mut self, buf: &mut [i16]) {
        let result = (self.nsf.GetSamples(buf.as_mut_ptr() as _, (buf.len()<<1) as _) as usize)>>1;
        // notsofatso may not fill the entire buffer...
        // in which case just repeat the last sample
        let l = buf[result-2];
        let r = buf[result-1];
        for i in (result>>1)..(buf.len()>>1) {
            buf[i<<1] = l;
            buf[(i<<1)|1] = r;
        }
    }

    pub fn write_sq1_vol(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4000, value);
    }

    pub fn write_sq1_sweep(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4001, value);
    }

    pub fn write_sq1_lo(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4002, value);
    }

    pub fn write_sq1_hi(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4003, value);
    }

    pub fn write_sq2_vol(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4004, value);
    }

    pub fn write_sq2_sweep(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4005, value);
    }

    pub fn write_sq2_lo(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4006, value);
    }

    pub fn write_sq2_hi(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4007, value);
    }

    pub fn write_tri_linear(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4008, value);
    }

    pub fn write_tri_lo(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x400a, value);
    }

    pub fn write_tri_hi(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x400b, value);
    }

    pub fn write_noise_vol(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x400c, value);
    }

    pub fn write_noise_lo(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x400e, value);
    }

    pub fn write_noise_hi(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x400f, value);
    }

    pub fn write_dmc_freq(&mut self, value: u8) {}

    pub fn write_dmc_raw(&mut self, value: u8) {}

    pub fn write_dmc_start(&mut self, value: u8) {}

    pub fn write_dmc_len(&mut self, value: u8) {}

    pub fn read_snd_chn(&mut self) -> u8 {
        self.nsf.ReadMemory_pAPU(0x4015)
    }

    pub fn write_snd_chn(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4015, value);
    }

    pub fn write_joy2(&mut self, value: u8) {
        self.nsf.WriteMemory_pAPU(0x4017, value);
    }
}