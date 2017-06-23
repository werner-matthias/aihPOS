#![allow(dead_code)]
use core::mem;
//use debug::blink;
//use debug::kprint::fkprint;

/// Siehe https://github.com/raspberrypi/firmware/wiki/Mailbox-property-interface

pub const BUFFER_SIZE: usize = 1024;

#[derive(Clone,Copy)]
#[repr(u32)]
pub enum Tag {
    None = 0,
    // Firmware
    GetFirmwareVersion = 0x1,

    // Board
    GetBoardModel = 0x10001,
    GetBoardRevision,
    GetBoardMacAddress,
    GetBoardSerial,
    GetArmMemory,
    GetVcMemory,
    GetClocks,

    // Befehlszeile
    GetCommandLine = 0x50001,

    // DMA
    GetDmaChannels = 0x60001,

    // Powermanagement
    GetPowerState = 0x20001,
    GetTiming = 0x20002,
    SetPowerState = 0x28001,

    // Uhren
    GetClockState = 0x30001,
    SetClockState = 0x38001,
    GetClockRate = 0x30002,
    SetClockRate = 0x38002,
    GetMaxClockRate = 0x30004,
    GetMinClockRate = 0x30007,
    GetTurbo = 0x30009,
    SetTurbo = 0x38009,

    // Spannungsregelung
    GetVoltage = 0x30003,
    SetVoltage = 0x38003,
    GetMaxVoltage = 0x30005,
    GetMinVoltage = 0x30008,
    GetTemperature = 0x30006,
    GetMaxTemperature = 0x3000A,
    AllocateMemory = 0x3000C,
    LockMemory = 0x3000D,
    UnlockMemory = 0x3000E,
    ReleaseMemory = 0x3000F,
    ExecuteCode = 0x30010,
    GetDispmanxResHandle = 0x30014,
    GetEDIDBlock = 0x30020,

    // Framebuffer
    AllocateFrameBuffer = 0x40001,
    ReleaseFrameBuffer = 0x48001,
    BlankScreen = 0x40002,
    GetPhysicalDisplaySize = 0x40003,
    TestPhysicalDisplaySize = 0x44003,
    SetPhysicalDisplaySize = 0x48003,
    GetVirtualDisplaySize = 0x40004,
    TestVirtualDisplaySize = 0x44004,
    SetVirtualDisplaySize = 0x48004,
    GetDepth = 0x40005,
    TestDepth = 0x44005,
    SetDepth = 0x48005,
    GetPixelOrder = 0x40006,
    TestPixelOrder = 0x44006,
    SetPixelOrder = 0x48006,
    GetAlphaMode = 0x40007,
    TestAlphaMode = 0x44007,
    SetAlphaMode = 0x48007,
    GetPitch = 0x40008,
    GetVirtualOffset = 0x40009,
    TestVirtualOffset = 0x44009,
    SetVirtualOffset = 0x48009,
    GetOverscan = 0x4000A,
    TestOverscan = 0x4400A,
    SetOverscan = 0x4800A,
    GetPalette = 0x4000B,
    TestPalette = 0x4400B,
    SetPalette = 0x4800B,
    SetCursorInfo = 0x8011,
    SetCursorState = 0x8010
}

struct ReqProperty {
    pub tag:  Tag,
    pub buf_size: usize,
    pub param_size: usize,
}

impl ReqProperty {
    fn new(tag: Tag) -> ReqProperty {
        let (buf_size, param_size) = // buf_size ist in Bytes, param_size in u32
            match tag {
                Tag::GetFirmwareVersion|
                Tag::GetBoardModel |
                Tag::GetBoardRevision |
                Tag::GetBoardMacAddress |
                Tag::GetBoardSerial |
                Tag::GetArmMemory |
                Tag::GetVcMemory |
                Tag::GetDmaChannels |
                Tag::GetPhysicalDisplaySize |
                Tag::GetVirtualDisplaySize |
                Tag::GetVirtualOffset
                => (8,0),
                Tag::GetClocks |
                Tag::GetCommandLine
                => (256,0),
                Tag::GetPowerState |
                Tag::GetTiming |
                Tag::GetClockState |
                Tag::GetClockRate |
                Tag::GetMaxClockRate |
                Tag::GetMinClockRate |
                Tag::GetTurbo |
                Tag::GetVoltage |
                Tag::GetMaxVoltage |
                Tag::GetMinVoltage |
                Tag::GetTemperature |
                Tag::GetMaxTemperature |
                Tag::GetDispmanxResHandle |
                Tag::AllocateFrameBuffer
                => (8,1),
                Tag::TestPhysicalDisplaySize |
                Tag::SetPhysicalDisplaySize |
                Tag::TestVirtualDisplaySize |
                Tag::SetVirtualDisplaySize |
                Tag::TestVirtualOffset |
                Tag::SetVirtualOffset |
                Tag::SetPowerState |
                Tag::SetClockState |
                Tag::SetTurbo |
                Tag::SetVoltage
                => (8,2),
                Tag::SetAlphaMode |
                Tag::SetPixelOrder |
                Tag::SetDepth |
                Tag::LockMemory |
                Tag::ReleaseMemory |
                Tag::UnlockMemory |
                Tag::BlankScreen |
                Tag::TestDepth |
                Tag::TestPixelOrder |
                Tag::TestAlphaMode
                => (4,1),
                Tag::GetAlphaMode |
                Tag::GetPixelOrder |
                Tag::GetPitch |
                Tag::GetDepth
                => (4,0),
                Tag::GetOverscan  
                => (16,0),
                Tag::SetOverscan |
                Tag::TestOverscan
                => (16,4),
                Tag::SetClockRate
                => (12,3),  // Antwortgröße: 8
                Tag::AllocateMemory
                => (12,3),  // Antwortgröße: 4
                Tag::ExecuteCode
                => (28,1),
                Tag::GetEDIDBlock
                => (136,1),
                Tag::ReleaseFrameBuffer
                => (0,0),
                Tag::GetPalette
                => (1024,0),
                Tag::TestPalette |
                Tag::SetPalette
                => (1032,258),  // maximale Größe
                Tag::SetCursorInfo
                => (24,6),
                Tag::SetCursorState
                => (16,4),
                Tag::None => (0,0)
        };
        ReqProperty{ tag: tag, buf_size: buf_size, param_size: param_size}
    }
}

#[repr(u32)]
enum ReqResCode {
    Request = 0,
    Success = 0x80000000,
    Error   = 0x80000001,
}

enum TagReqResp {
    Request = 0,
    Response = 1 << 31
}

enum PbOffset {
    Size = 0,
    Code = 1
}

enum TagOffset {
    Id = 0,
    Size = 1,
    ReqResp = 2,
    StartVal = 3,
}

#[repr(C)]
#[repr(align(16))]
pub struct PropertyTagBuffer {
    pub data: [u32; BUFFER_SIZE],
    pub index:    usize,
}

impl PropertyTagBuffer {

    pub fn new() -> PropertyTagBuffer {
        PropertyTagBuffer{
            data: [0; BUFFER_SIZE],
            index: 2,
        }
    }

    pub fn init(&mut self)  {
        self.index = 2;
        self.data[PbOffset::Size as usize] = 12; // Size + Code + Endtag
        self.data[PbOffset::Code as usize] = ReqResCode::Request as u32;
        self.data[self.index] = Tag::None as u32;
    }

    fn write(&mut self, data: u32){
        self.data[self.index] = data;
        self.data[PbOffset::Size as usize] += mem::size_of::<u32> as u32;
        self.index += 1;
    }

    pub fn add_tag_with_param(&mut self, tag: Tag,  params: Option<&[u32]>) {
        let old_index = self.index;
        let prop = ReqProperty::new(tag);
        self.write(prop.tag as u32);
        self.write(prop.buf_size as u32);
        self.write(TagReqResp::Request as u32);
        match params {
            Some(array) => {
                assert!(array.len() == prop.param_size); // ToDo: Überprüfung zur Compilezeit wäre besser
                for p in array.into_iter() {
                    self.write(*p);
                }
            },
            None => {}
        }
        self.index = old_index + 3 + (prop.buf_size >> 2) ;
        self.data[self.index] = Tag::None as u32; 
        self.data[PbOffset::Size as usize] = ((self.index +1) << 2) as u32; 
    }

    fn read(&mut self) -> u32 {
        self.index + 1;
        self.data[self.index - 1]
    }

    pub fn get_answer(&self, tag: Tag) -> Option<&[u32]> {
        if self.data[PbOffset::Code as usize] != ReqResCode::Success as u32 {
            return None
        }
        // Es wird ein lokaler Index benutzt, so dass der PropertyTagBuffer nicht geändert wird
        let mut index: usize = 2;

        while (index as u32) < (self.data[PbOffset::Size as usize]) {
            if (self.data[index] == tag as u32) &&
                (self.data[index + TagOffset::ReqResp as usize] & TagReqResp::Response as u32 == TagReqResp::Response as u32) {
                let to = index + TagOffset::StartVal as usize +
                    ((self.data[index + TagOffset::Size as usize] & !(TagReqResp::Response as u32)) >> 2) as usize;
                let ret = self.data.get(index + TagOffset::StartVal as usize .. to);
                return ret;
            }
            index += (self.data[index+TagOffset::Size as usize] >> 2) as usize + 3;
        }
        None
    }
}
