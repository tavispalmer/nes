#![allow(non_camel_case_types)]
#![allow(non_snake_case)]

use std::{ffi::c_void, mem::MaybeUninit};

// define c++ allocation functions
// so we don't have to link with stdc++
extern "C" {
    fn malloc(size: usize) -> *mut c_void;
    fn free(ptr: *mut c_void);
}

// operator new[](unsigned long)
#[no_mangle]
pub extern "C" fn _Znam(count: usize) -> *mut c_void {
    unsafe {
        malloc(count)
    }
}

// operator delete[](void*)
#[no_mangle]
pub extern "C" fn _ZdaPv(ptr: *mut c_void) {
    unsafe {
        free(ptr)
    }
}

#[repr(C)]
pub struct CNSFFile {
    pub bIsExtended: bool,
    pub nIsPal: u8,
    pub nLoadAddress: i32,
    pub nInitAddress: i32,
    pub nPlayAddress: i32,
    pub nChipExtensions: u8,
    
    pub nNTSC_PlaySpeed: i32,
    pub nPAL_PlaySpeed: i32,

    pub nTrackCount: i32,
    pub nInitialTrack: i32,

    pub pDataBuffer: *mut u8,
    pub nDataBufferSize: i32,

    pub pPlaylist: *mut u8,
    pub nPlaylistSize: i32,

    pub pTrackTime: *mut i32,
    pub pTrackFade: *mut i32,

    pub szTrackLabels: *mut *mut i8,

    pub szGameTitle: *mut i8,
    pub szArtist: *mut i8,
    pub szCopyright: *mut i8,
    pub szRipper: *mut i8,

    pub nBankswitch: [u8; 8],
}

impl CNSFFile {
    #[inline]
    pub fn new() -> CNSFFile {
        unsafe {
            MaybeUninit::zeroed().assume_init()
        }
    }

    pub fn Destroy(&mut self) {
        unsafe {
            _ZN8CNSFFile7DestroyEv(self)
        }
    }
}

impl Drop for CNSFFile {
    #[inline]
    fn drop(&mut self) {
        self.Destroy()
    }
}

extern "C" {
    fn _ZN8CNSFFile7DestroyEv(this: *mut CNSFFile);
}

#[repr(C)]
pub struct TWIN(u16);

#[repr(C)]
pub struct QUAD(u32);

// Wave_Square.h
#[repr(C)]
pub struct CSquareWaves {
    pub nFreqTimer: [TWIN; 2],
    pub nFreqCount: [i32; 2],

    pub nLengthCount: [u8; 2],
    pub bLengthEnabled: [u8; 2],
    pub bChannelEnabled: [u8; 2],

    pub nVolume: [u8; 2],
    pub nDecayVolume: [u8; 2],
    pub bDecayEnable: [u8; 2],
    pub bDecayLoop: [u8; 2],
    pub nDecayTimer: [u8; 2],
    pub nDecayCount: [u8; 2],

    pub bSweepEnable: [u8; 2],
    pub bSweepMode: [u8; 2],
    pub bSweepForceSilence: [u8; 2],
    pub nSweepTimer: [u8; 2],
    pub nSweepCount: [u8; 2],
    pub nSweepShift: [u8; 2],

    pub nDutyCount: [u8; 2],
    pub nDutyCycle: [u8; 2],

    pub bChannelMix: [u8; 2],
    pub nOutputTable_L: [i16; 0x100],
    pub nOutputTable_R: [[i16; 0x100]; 3],
    pub nMixL: i32,
    pub nMixR: i32,

    pub bInvert: u8,
    pub bDoInvert: u8,
    pub nInvertFreqCutoff: u16,
}

#[repr(C)]
pub struct CTNDWaves {
    pub nTriFreqTimer: TWIN,
    pub nTriFreqCount: i32,

    pub nTriLengthCount: u8,
    pub bTriLengthEnabled: u8,
    pub bTriChannelEnabled: u8,

    pub nTriLinearCount: u8,
    pub nTriLinearLoad: u8,
    pub bTriLinearHalt: u8,
    pub bTriLinearControl: u8,

    pub nTriStep: u8,
    pub nTriOutput: u8,
    pub bTriChannelMix: u8,

    pub nNoiseFreqTimer: u16,
    pub nNoiseFreqCount: i32,

    pub nNoiseLengthCount: u8,
    pub bNoiseLengthEnabled: u8,
    pub bNoiseChannelEnabled: u8,

    pub nNoiseVolume: u8,
    pub nNoiseDecayVolume: u8,
    pub bNoiseDecayEnable: u8,
    pub bNoiseDecayLoop: u8,
    pub nNoiseDecayTimer: u8,
    pub nNoiseDecayCount: u8,

    pub nNoiseRandomShift: u16,
    pub bNoiseRandomMode: u8,
    pub bNoiseRandomOut: u8,
    pub bNoiseChannelMix: u8,

    pub bDMCLoop: u8,
    pub bDMCIRQEnabled: u8,
    pub bDMCIRQPending: u8,

    pub nDMCDMABank_Load: u8,
    pub nDMCDMAAddr_Load: u16,
    pub nDMCDMABank: u8,
    pub nDMCDMAAddr: u16,
    pub pDMCDMAPtr: [*mut u8; 8],

    pub nDMCLength: u16,
    pub nDMCBytesRemaining: u16,
    pub nDMCDelta: u8,
    pub nDMCDeltaBit: u8,
    pub bDMCDeltaSilent: u8,
    pub nDMCSampleBuffer: u8,
    pub bDMCSampleBufferEmpty: u8,

    pub nDMCFreqTimer: u16,
    pub nDMCFreqCount: u16,

    pub bDMCActive: u8,
    pub nDMCOutput: u8,
    pub bDMCChannelMix: u8,

    pub nOutputTable_L: *mut i16,
    pub nOutputTable_R: *mut i16,
    pub nMixL: i32,
    pub nMixR: i32,

    pub bInvert: u8,
    pub bDoInvert: u8,
    pub nInvertFreqCutoff_Noise: u16,
    pub nInvertFreqCutoff_Tri: u16,
}

#[repr(C)]
pub struct CVRC6PulseWave {
    pub nFreqTimer: TWIN,
    pub nFreqCount: i32,
    pub nFreqInvHalt: i32,

    pub bChannelEnabled: u8,
    pub bDigitized: u8,

    pub nVolume: u8,

    pub nDutyCycle: u8,
    pub nDutyCount: u8,

    pub nOutputTable_L: [i16; 0x10],
    pub nOutputTable_R: [i16; 0x10],
    pub nMixL: i32,
    pub nMixR: i32,

    pub bInvert: u8,
    pub bDoInvert: u8,
    pub nInvertFreqCutoff: u16,
}

#[repr(C)]
pub struct CVRC6SawWave {
    pub nFreqTimer: TWIN,
    pub nFreqCount: i32,

    pub bChannelEnabled: u8,
    pub nAccumRate: u8,
    pub nAccum: u8,
    pub nAccumStep: u8,

    pub nOutputTable_L: [i16; 0x20],
    pub nOutputTable_R: [i16; 0x20],
    pub nMixL: i32,
    pub nMixR: i32,

    pub bInvert: u8,
    pub bDoInvert: u8,
    pub nInvertFreqCutoff: u16,
}

#[repr(C)]
pub struct CMMC5SquareWave {
    pub nFreqTimer: TWIN,
    pub nFreqCount: i32,

    pub nLengthCount: u8,
    pub bLengthEnabled: u8,
    pub bChannelEnabled: u8,

    pub nVolume: u8,
    pub nDecayVolume: u8,
    pub bDecayEnable: u8,
    pub bDecayLoop: u8,
    pub nDecayTimer: u8,
    pub nDecayCount: u8,

    pub nDutyCount: u8,
    pub nDutyCycle: u8,

    pub bChannelMix: u8,
    pub nOutputTable_L: [i16; 0x10],
    pub nOutputTable_R: [i16; 0x10],
    pub nMixL: i32,
    pub nMixR: i32,

    pub bDoInvert: u8,
    pub bInvert: u8,
    pub nInvertFreqCutoff: u16,
}

#[repr(C)]
pub struct CMMC5VoiceWave {
    pub nOutput: u8,
    pub nOutputTable_L: [i16; 0x80],
    pub nOutputTable_R: [i16; 0x80],
    pub nMixL: i32,
    pub nMixR: i32,
    pub bInvert: u8,
}

#[repr(C)]
pub struct CN106Wave {
    pub nActiveChannels: u8,
    pub bAutoIncrement: u8,
    pub nCurrentAddress: u8,
    pub nRAM: [u8; 0x100],
    pub fFrequencyLookupTable: [f32; 8],

    pub nFreqReg: [QUAD; 8],
    pub fFreqTimer: [f32; 8],
    pub fFreqCount: [f32; 8],

    pub nWaveSize: [u8; 8],
    pub nWaveRemaining: [u8; 8],

    pub nWavePosStart: [u8; 8],
    pub nWavePos: [u8; 8],
    pub nOutput: [u8; 8],

    pub nVolume: [u8; 8],

    pub nPreVolume: [u8; 8],
    pub nPopCheck: [u8; 8],

    pub nOutputTable_L: [[[i16; 0x10]; 0x10]; 8],
    pub nOutputTable_R: [[[i16; 0x10]; 0x10]; 8],

    pub nMixL: [i32; 8],
    pub nMixR: [i32; 8],

    pub bInvert: [u8; 8],
    pub bDoInvert: [u8; 8],
    pub nInvertFreqCutoff: [[u32; 8]; 8],
    pub nWaveSizeWritten: [u8; 8],
    pub nInvCheck: [u8; 8],
}

#[repr(C)]
pub struct CFDSWave {
    pub bEnvelopeEnable: u8,
    pub nEnvelopeSpeed: u8,

    pub nVolEnv_Mode: u8,
    pub nVolEnv_Decay: u8,
    pub nVolEnv_Gain: u8,
    pub nVolEnv_Timer: i32,
    pub nVolEnv_Count: i32,
    pub nVolume: u8,
    pub bVolEnv_On: u8,

    pub nSweep_Mode: u8,
    pub nSweep_Decay: u8,
    pub nSweep_Timer: i32,
    pub nSweep_Count: i32,
    pub nSweep_Gain: u8,
    pub bSweepEnv_On: u8,

    pub nSweepBias: i32,
    pub bLFO_Enabled: u8,
    pub nLFO_Freq: TWIN,
    pub fLFO_Timer: f32,
    pub fLFO_Count: f32,
    pub nLFO_Addr: u8,
    pub nLFO_Table: [u8; 0x40],
    pub bLFO_On: u8,

    pub nMainVolume: u8,
    pub bEnabled: u8,
    pub nFreq: TWIN,
    pub fFreqCount: f32,
    pub nMainAddr: u8,
    pub nWaveTable: [u8; 0x40],
    pub bWaveWrite: u8,
    pub bMain_On: u8,

    pub nOutputTable_L: [[[i16; 0x40]; 0x21]; 4],
    pub nOutputTable_R: [[[i16; 0x40]; 0x21]; 4],
    pub nMixL: i32,
    pub nMixR: i32,

    pub bInvert: u8,
    pub bPopReducer: u8,
    pub nPopOutput: u8,
    pub nPopCount: i32,
}

#[repr(C)]
pub struct CFME07Wave {
    pub nFreqTimer: TWIN,
    pub nFreqCount: i32,

    pub bChannelEnabled: u8,

    pub nVolume: u8,

    pub nDutyCount: u8,

    pub nOutputTable_L: [i16; 0x10],
    pub nOutputTable_R: [i16; 0x10],
    pub nMixL: i32,
    pub nMixR: i32,

    pub bInvert: u8,
    pub bDoInvert: u8,
    pub nInvertFreqCutoff: u16,
}

// NSFCore

#[repr(C)]
pub struct CNSFCore {
    pub bMemoryOK: u8,
    pub bFileLoaded: u8,
    pub bTrackSelected: u8,
    pub bIsGeneratingSamples: u8, // volatile

    pub pRAM: *mut u8,
    pub pSRAM: *mut u8,
    pub pExRAM: *mut u8,

    pub pROM_Full: *mut u8,

    pub pROM: [*mut u8; 10],

    pub pStack: *mut u8,

    pub nROMSize: i32,
    pub nROMBankCount: i32,
    pub nROMMaxSize: i32,

    pub ReadMemory: [(usize, usize); 0x10],
    pub WriteMemory: [(usize, usize); 0x10],

    pub regA: u8,
    pub regX: u8,
    pub regY: u8,
    pub regP: u8,
    pub regSP: u8,
    pub regPC: u16,

    pub pPALMode: u8,
    pub bCPUJammed: u8,

    pub nMultIn_Low: u8,
    pub nMultIn_High: u8,

    pub nBankswitchInitValues: [u8; 10],
    pub nPlayAddress: u16,
    pub nInitAddress: u16,

    pub nExternalSound: u8,
    pub nCurTrack: u8,
    
    pub fNSFPlaybackSpeed: f32,

    pub nFrameCounter: u8,
    pub nFrameCounterMax: u8,
    pub bFrameIRQEnabled: u8,
    pub bFrameIRQPending: u8,

    pub bChannelMix: [u8; 24],
    pub nChannelVol: [u8; 29],
    pub nChannelPan: [i8; 29],

    pub mWave_Squares: CSquareWaves,
    pub mWave_TND: CTNDWaves,
    pub mWave_VRC6Pulse: [CVRC6PulseWave; 2],
    pub mWave_VRC6Saw: CVRC6SawWave,
    pub mWave_MMC5Square: [CMMC5SquareWave; 2],
    pub mWave_MMC5Voice: CMMC5VoiceWave,
    pub mWave_N106: CN106Wave,
    pub mWave_FDS: CFDSWave,

    pub nFME07_Address: u8,
    pub mWave_FME07: [CFME07Wave; 3],

    pub nVRC7Address: u8,
    pub pVRC7Buffer: *mut u8,
    pub pFMOPL: *mut c_void,
    pub VRC7Chan: [[u8; 6]; 3],
    pub bVRC7_FadeChanged: u8,
    pub bVRC7Inv: [u8; 6],

    pub fTicksUntilNextFrame: f32,

    pub fTicksPerPlay: f32,
    pub fTicksUntilNextPlay: f32,

    pub fTicksPerSample: f32,
    pub fTicksUntilNextSample: f32,

    pub nCPUCycle: u32,
    pub nAPUCycle: u32,
    pub nTotalPlays: u32,

    pub nSilentSamples: i32,
    pub nSilentSampleMax: i32,
    pub nSilenceTrackMS: i32,
    pub bNoSilenceIfTime: u8,
    pub bTimeNotDefault: u8,

    pub nSampleRate: i32,
    pub nMonoStereo: i32,

    pub fMasterVolume: f32,

    pub nStartFade: u32,
    pub nEndFade: u32,
    pub bFade: u8,
    pub fFadeVolume: f32,
    pub fFadeChange: f32,

    pub pOutput: *mut u8,
    pub nDownsample: i32,

    pub bDMCPopReducer: u8,
    pub nDMCPop_Prev: u8,
    pub bDMCPop_Skip: u8,
    pub bDMCPop_SamePlay: u8,

    pub nForce4017Write: u8,
    pub bN106PopReducer: u8,
    pub nInvertCutoffHz: i32,
    pub bIgnore4011Writes: u8,

    pub bIgnoreBRK: u8,
    pub bIgnoreIllegalOps: u8,
    pub bNoWaitForReturn: u8,
    pub bPALPreference: u8,
    pub bCleanAXY: u8,
    pub bResetDuty: u8,

    pub nFilterAccL: i64,
    pub nFilterAccR: i64,
    pub nFilterAcc2L: i64,
    pub nFilterAcc2R: i64,
    pub nHighPass: i64,
    pub nLowPass: i64,

    pub nHighPassBase: i32,
    pub nLowPassBase: i32,

    pub bHighPassEnabled: u8,
    pub bLowPassEnabled: u8,
    pub bPrePassEnabled: u8,

    pub nSmPrevL: i32,
    pub nSmAccL: i32,
    pub nSmPrevR: i32,
    pub nSmAccR: i32,
    pub nPrePassBase: i32,
    pub fSmDiv: f32,
}

impl CNSFCore {
    #[inline]
    pub fn new() -> CNSFCore {
        unsafe {
            let mut this = MaybeUninit::uninit();
            _ZN8CNSFCoreC1Ev(this.as_mut_ptr() as _);
            this.assume_init()
        }
    }
    #[inline]
    pub fn Initialize(&mut self) -> i32 {
        unsafe {
            _ZN8CNSFCore10InitializeEv(self)
        }
    }
    #[inline]
    pub fn Destroy(&mut self) {
        unsafe {
            _ZN8CNSFCore7DestroyEv(self)
        }
    }
    #[inline]
    pub fn LoadNSF(&mut self, file: &CNSFFile) -> i32 {
        unsafe {
            _ZN8CNSFCore7LoadNSFEPK8CNSFFile(self, file)
        }
    }
    #[inline]
    pub fn SetTrack(&mut self, track: u8) {
        unsafe {
            _ZN8CNSFCore8SetTrackEh(self, track)
        }
    }
    #[inline]
    pub fn GetSamples(&mut self, buffer: *mut u8, buffersize: i32) -> i32 {
        unsafe {
            _ZN8CNSFCore10GetSamplesEPhi(self, buffer, buffersize)
        }
    }
    #[inline]
    pub fn SetPlaybackOptions(&mut self, samplerate: i32, channels: i32) -> i32 {
        unsafe {
            _ZN8CNSFCore18SetPlaybackOptionsEii(self, samplerate, channels)
        }
    }
    #[inline]
    pub fn SetChannelOptions(&mut self, chan: u32, mix: i32, vol: i32, pan: i32, inv: i32) {
        unsafe {
            _ZN8CNSFCore17SetChannelOptionsEjiiii(self, chan, mix, vol, pan, inv)
        }
    }
    #[inline]
    pub fn ReadMemory_pAPU(&mut self, a: u16) -> u8 {
        unsafe {
            _ZN8CNSFCore15ReadMemory_pAPUEt(self, a)
        }
    }
    #[inline]
    pub fn WriteMemory_pAPU(&mut self, a: u16, v: u8) {
        unsafe {
            _ZN8CNSFCore16WriteMemory_pAPUEth(self, a, v)
        }
    }
}

impl Drop for CNSFCore {
    #[inline]
    fn drop(&mut self) {
        self.Destroy();
    }
}

extern "C" {
    fn _ZN8CNSFCoreC1Ev(this: *mut c_void) -> *mut CNSFCore;
    fn _ZN8CNSFCore10InitializeEv(this: *mut CNSFCore) -> i32;
    fn _ZN8CNSFCore7DestroyEv(this: *mut CNSFCore);
    fn _ZN8CNSFCore7LoadNSFEPK8CNSFFile(this: *mut CNSFCore, file: *const CNSFFile) -> i32;
    fn _ZN8CNSFCore8SetTrackEh(this: *mut CNSFCore, track: u8);
    fn _ZN8CNSFCore10GetSamplesEPhi(this: *mut CNSFCore, buffer: *mut u8, buffersize: i32) -> i32;
    fn _ZN8CNSFCore18SetPlaybackOptionsEii(this: *mut CNSFCore, samplerate: i32, channels: i32) -> i32;
    fn _ZN8CNSFCore17SetChannelOptionsEjiiii(this: *mut CNSFCore, chan: u32, mix: i32, vol: i32, pan: i32, inv: i32);
    fn _ZN8CNSFCore15ReadMemory_pAPUEt(this: *mut CNSFCore, a: u16) -> u8;
    fn _ZN8CNSFCore16WriteMemory_pAPUEth(this: *mut CNSFCore, a: u16, v: u8);
}
