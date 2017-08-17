mod mailbox;
mod propertytags;
pub use self::propertytags::{Tag,PropertyTagBuffer,BUFFER_SIZE};
pub use hal::board::mailbox::{mailbox, Channel};
use memory::Address;

pub enum BoardReport {
    FirmwareVersion,
    BoardModel,
    BoardRevision,
    SerialNumber
}

pub fn report_board_info(kind: BoardReport) -> u32 {  
    let mut prob_tag_buf: PropertyTagBuffer = PropertyTagBuffer::new();
    prob_tag_buf.init();
    let tag = match kind {
        BoardReport::FirmwareVersion => Tag::GetFirmwareVersion,
        BoardReport::BoardModel      => Tag::GetBoardModel,
        BoardReport::BoardRevision   => Tag::GetBoardRevision,
        BoardReport::SerialNumber    => Tag::GetBoardSerial
    };
    prob_tag_buf.add_tag_with_param(tag,None);
    let mb = mailbox(0);
    mb.write(Channel::ATags, &prob_tag_buf.data as *const [u32; self::propertytags::BUFFER_SIZE] as u32);
    mb.read(Channel::ATags);
    match prob_tag_buf.get_answer(tag) {
        Some(n) => n[0],
        _       => 0
    }
}

#[allow(dead_code)]
pub enum MemReport {
    ArmStart,
    ArmSize,
    VcStart,
    VcSize,
}

pub fn report_memory(kind: MemReport) -> Address {
    let mut prob_tag_buf = PropertyTagBuffer::new();
    prob_tag_buf.init();
    let tag = match kind {
        MemReport::ArmStart | MemReport::ArmSize => Tag::GetArmMemory,
        MemReport::VcStart  | MemReport::VcSize  => Tag::GetVcMemory
    };    
    prob_tag_buf.add_tag_with_param(tag,None);
    let mb = mailbox(0);
    mb.write(Channel::ATags, &prob_tag_buf.data as *const [u32; self::propertytags::BUFFER_SIZE] as u32);
    mb.read(Channel::ATags);
    let array = prob_tag_buf.get_answer(tag);
    match array {
        Some(a) => match kind {
            MemReport::ArmStart | MemReport::VcStart => a[0] as Address,
            MemReport::ArmSize  | MemReport::VcSize  => a[1] as Address
        },
        None => 0
    }
}
