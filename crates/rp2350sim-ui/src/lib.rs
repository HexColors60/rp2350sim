//! RP2350 UI System

mod ui;
mod dialogs;
pub mod panels;

pub use ui::{Ui, UiState, UiEvent, WaveformSignal};
pub use dialogs::{
    FileDialog, FileDialogState, FileDialogMode, FileDialogResult,
    FirmwareLoaderDialog, SaveStateDialog, ProjectOpenDialog,
};
pub use panels::{
    GpioPanel, UartPanel, SpiPanel, I2cPanel, PioPanel, AdcPwmPanel, TimerPanel, UsbPanel,
    DmaPanel, DmaState, DmaChannelState, XipPanel, XipState,
    PeripheralState, PeripheralEvent, PeripheralPanelManager, PeripheralTab,
};