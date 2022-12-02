use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(u32)]
pub enum ReceiverError {
    InitFailed = 1,
    ExecOpFailed = 2,
}
