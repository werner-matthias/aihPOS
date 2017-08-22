mod mailbox;
mod propertytags;

pub use self::propertytags::{Tag,PropertyTagBuffer,BUFFER_SIZE};
pub use self::mailbox::{mailbox, Channel};

/// Art der verlangten Information
pub enum BoardReport {
    /// Version der Firmware
    FirmwareVersion,
    /// Code für den Computertyp (sollte 0 = Raspberry Pi sein)
    BoardModel,
    /// Version des Raspberrys
    BoardRevision,
    /// Seriennummer
    SerialNumber
}

/// Gibt Informationen über die Hardware
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
    mb.write(Channel::ATags, prob_tag_buf.data_addr() as u32);
    mb.read(Channel::ATags);
    match prob_tag_buf.get_answer(tag) {
        Some(n) => n[0],
        _       => 0
    }
}

/// 
#[allow(dead_code)]
pub enum MemReport {
    /// Beginn des ARM-Speicherbereiches
    ArmStart,
    /// Größe des ARM-Speicherbereiches
    ArmSize,
    /// Beginn des Speicherbereiches des Videoprozessors
    VcStart,
    /// Größe des Speicherbereiches des Videoprozessors
    VcSize,
}

/// Gibt Informationen über die Speicheraufteilung
pub fn report_memory(kind: MemReport) -> usize {
    let mut prob_tag_buf = PropertyTagBuffer::new();
    prob_tag_buf.init();
    let tag = match kind {
        MemReport::ArmStart | MemReport::ArmSize => Tag::GetArmMemory,
        MemReport::VcStart  | MemReport::VcSize  => Tag::GetVcMemory
    };    
    prob_tag_buf.add_tag_with_param(tag,None);
    let mb = mailbox(0);
    mb.write(Channel::ATags, prob_tag_buf.data_addr() as u32);
    mb.read(Channel::ATags);
    let array = prob_tag_buf.get_answer(tag);
    match array {
        Some(a) => match kind {
            MemReport::ArmStart | MemReport::VcStart => a[0] as usize,
            MemReport::ArmSize  | MemReport::VcSize  => a[1] as usize
        },
        None => 0
    }
}
