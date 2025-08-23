// External Commands Registry
// Professional command system with comprehensive utilities

pub mod filesystem;
pub mod network;
pub mod system;
pub mod process;
pub mod hardware;
pub mod development;
pub mod multimedia;
pub mod security;
pub mod bluetooth;
pub mod wifi;

use alloc::{string::String, vec::Vec, collections::BTreeMap};
use crate::shell::{Shell, ShellResult, SimpleCommand};

/// External command trait
pub trait ExternalCommand {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn usage(&self) -> &'static str;
    fn execute(&self, shell: &mut Shell, cmd: &SimpleCommand) -> ShellResult<i32>;
}

/// Command registry
static mut EXTERNAL_COMMANDS: Option<BTreeMap<String, Box<dyn ExternalCommand>>> = None;

/// Initialize external commands
pub fn init_commands() {
    let mut commands: BTreeMap<String, Box<dyn ExternalCommand>> = BTreeMap::new();
    
    // Filesystem commands
    commands.insert("ls".to_string(), Box::new(filesystem::LsCommand));
    commands.insert("cat".to_string(), Box::new(filesystem::CatCommand));
    commands.insert("cp".to_string(), Box::new(filesystem::CpCommand));
    commands.insert("mv".to_string(), Box::new(filesystem::MvCommand));
    commands.insert("rm".to_string(), Box::new(filesystem::RmCommand));
    commands.insert("mkdir".to_string(), Box::new(filesystem::MkdirCommand));
    commands.insert("rmdir".to_string(), Box::new(filesystem::RmdirCommand));
    commands.insert("touch".to_string(), Box::new(filesystem::TouchCommand));
    commands.insert("find".to_string(), Box::new(filesystem::FindCommand));
    commands.insert("grep".to_string(), Box::new(filesystem::GrepCommand));
    commands.insert("head".to_string(), Box::new(filesystem::HeadCommand));
    commands.insert("tail".to_string(), Box::new(filesystem::TailCommand));
    commands.insert("wc".to_string(), Box::new(filesystem::WcCommand));
    commands.insert("sort".to_string(), Box::new(filesystem::SortCommand));
    commands.insert("uniq".to_string(), Box::new(filesystem::UniqCommand));
    commands.insert("cut".to_string(), Box::new(filesystem::CutCommand));
    commands.insert("awk".to_string(), Box::new(filesystem::AwkCommand));
    commands.insert("sed".to_string(), Box::new(filesystem::SedCommand));
    commands.insert("tar".to_string(), Box::new(filesystem::TarCommand));
    commands.insert("gzip".to_string(), Box::new(filesystem::GzipCommand));
    commands.insert("gunzip".to_string(), Box::new(filesystem::GunzipCommand));
    commands.insert("zip".to_string(), Box::new(filesystem::ZipCommand));
    commands.insert("unzip".to_string(), Box::new(filesystem::UnzipCommand));
    commands.insert("df".to_string(), Box::new(filesystem::DfCommand));
    commands.insert("du".to_string(), Box::new(filesystem::DuCommand));
    commands.insert("mount".to_string(), Box::new(filesystem::MountCommand));
    commands.insert("umount".to_string(), Box::new(filesystem::UmountCommand));
    commands.insert("fsck".to_string(), Box::new(filesystem::FsckCommand));
    commands.insert("mkfs".to_string(), Box::new(filesystem::MkfsCommand));
    commands.insert("lsblk".to_string(), Box::new(filesystem::LsblkCommand));
    commands.insert("fdisk".to_string(), Box::new(filesystem::FdiskCommand));
    
    // Network commands
    commands.insert("ping".to_string(), Box::new(network::PingCommand));
    commands.insert("wget".to_string(), Box::new(network::WgetCommand));
    commands.insert("curl".to_string(), Box::new(network::CurlCommand));
    commands.insert("ssh".to_string(), Box::new(network::SshCommand));
    commands.insert("scp".to_string(), Box::new(network::ScpCommand));
    commands.insert("rsync".to_string(), Box::new(network::RsyncCommand));
    commands.insert("netstat".to_string(), Box::new(network::NetstatCommand));
    commands.insert("ss".to_string(), Box::new(network::SsCommand));
    commands.insert("iptables".to_string(), Box::new(network::IptablesCommand));
    commands.insert("ip".to_string(), Box::new(network::IpCommand));
    commands.insert("ifconfig".to_string(), Box::new(network::IfconfigCommand));
    commands.insert("route".to_string(), Box::new(network::RouteCommand));
    commands.insert("nslookup".to_string(), Box::new(network::NslookupCommand));
    commands.insert("dig".to_string(), Box::new(network::DigCommand));
    commands.insert("host".to_string(), Box::new(network::HostCommand));
    commands.insert("traceroute".to_string(), Box::new(network::TracerouteCommand));
    commands.insert("mtr".to_string(), Box::new(network::MtrCommand));
    commands.insert("nc".to_string(), Box::new(network::NetcatCommand));
    commands.insert("telnet".to_string(), Box::new(network::TelnetCommand));
    commands.insert("ftp".to_string(), Box::new(network::FtpCommand));
    commands.insert("sftp".to_string(), Box::new(network::SftpCommand));
    
    // WiFi commands
    commands.insert("wifi".to_string(), Box::new(wifi::WifiCommand));
    commands.insert("iwconfig".to_string(), Box::new(wifi::IwconfigCommand));
    commands.insert("iwlist".to_string(), Box::new(wifi::IwlistCommand));
    commands.insert("wpa_cli".to_string(), Box::new(wifi::WpaCliCommand));
    commands.insert("wpa_supplicant".to_string(), Box::new(wifi::WpaSupplicantCommand));
    commands.insert("hostapd".to_string(), Box::new(wifi::HostapdCommand));
    commands.insert("aircrack-ng".to_string(), Box::new(wifi::AircrackCommand));
    commands.insert("airodump-ng".to_string(), Box::new(wifi::AirodumpCommand));
    commands.insert("aireplay-ng".to_string(), Box::new(wifi::AireplayCommand));
    
    // Bluetooth commands
    commands.insert("bluetooth".to_string(), Box::new(bluetooth::BluetoothCommand));
    commands.insert("bluetoothctl".to_string(), Box::new(bluetooth::BluetoothctlCommand));
    commands.insert("hciconfig".to_string(), Box::new(bluetooth::HciconfigCommand));
    commands.insert("hcitool".to_string(), Box::new(bluetooth::HcitoolCommand));
    commands.insert("hcidump".to_string(), Box::new(bluetooth::HcidumpCommand));
    commands.insert("rfcomm".to_string(), Box::new(bluetooth::RfcommCommand));
    commands.insert("sdptool".to_string(), Box::new(bluetooth::SdptoolCommand));
    commands.insert("obexftp".to_string(), Box::new(bluetooth::ObexftpCommand));
    commands.insert("bluez-test".to_string(), Box::new(bluetooth::BluezTestCommand));
    
    // System commands
    commands.insert("ps".to_string(), Box::new(system::PsCommand));
    commands.insert("top".to_string(), Box::new(system::TopCommand));
    commands.insert("htop".to_string(), Box::new(system::HtopCommand));
    commands.insert("kill".to_string(), Box::new(system::KillCommand));
    commands.insert("killall".to_string(), Box::new(system::KillallCommand));
    commands.insert("jobs".to_string(), Box::new(system::JobsCommand));
    commands.insert("bg".to_string(), Box::new(system::BgCommand));
    commands.insert("fg".to_string(), Box::new(system::FgCommand));
    commands.insert("nohup".to_string(), Box::new(system::NohupCommand));
    commands.insert("screen".to_string(), Box::new(system::ScreenCommand));
    commands.insert("tmux".to_string(), Box::new(system::TmuxCommand));
    commands.insert("systemctl".to_string(), Box::new(system::SystemctlCommand));
    commands.insert("service".to_string(), Box::new(system::ServiceCommand));
    commands.insert("crontab".to_string(), Box::new(system::CrontabCommand));
    commands.insert("at".to_string(), Box::new(system::AtCommand));
    commands.insert("uptime".to_string(), Box::new(system::UptimeCommand));
    commands.insert("who".to_string(), Box::new(system::WhoCommand));
    commands.insert("w".to_string(), Box::new(system::WCommand));
    commands.insert("last".to_string(), Box::new(system::LastCommand));
    commands.insert("id".to_string(), Box::new(system::IdCommand));
    commands.insert("groups".to_string(), Box::new(system::GroupsCommand));
    commands.insert("su".to_string(), Box::new(system::SuCommand));
    commands.insert("sudo".to_string(), Box::new(system::SudoCommand));
    commands.insert("passwd".to_string(), Box::new(system::PasswdCommand));
    commands.insert("useradd".to_string(), Box::new(system::UseraddCommand));
    commands.insert("userdel".to_string(), Box::new(system::UserdelCommand));
    commands.insert("usermod".to_string(), Box::new(system::UsermodCommand));
    commands.insert("groupadd".to_string(), Box::new(system::GroupaddCommand));
    commands.insert("groupdel".to_string(), Box::new(system::GroupdelCommand));
    commands.insert("chmod".to_string(), Box::new(system::ChmodCommand));
    commands.insert("chown".to_string(), Box::new(system::ChownCommand));
    commands.insert("chgrp".to_string(), Box::new(system::ChgrpCommand));
    commands.insert("umask".to_string(), Box::new(system::UmaskCommand));
    
    // Hardware commands
    commands.insert("lscpu".to_string(), Box::new(hardware::LscpuCommand));
    commands.insert("lsmem".to_string(), Box::new(hardware::LsmemCommand));
    commands.insert("lspci".to_string(), Box::new(hardware::LspciCommand));
    commands.insert("lsusb".to_string(), Box::new(hardware::LsusbCommand));
    commands.insert("lshw".to_string(), Box::new(hardware::LshwCommand));
    commands.insert("dmidecode".to_string(), Box::new(hardware::DmidecodeCommand));
    commands.insert("hwinfo".to_string(), Box::new(hardware::HwinfoCommand));
    commands.insert("sensors".to_string(), Box::new(hardware::SensorsCommand));
    commands.insert("smartctl".to_string(), Box::new(hardware::SmartctlCommand));
    commands.insert("hdparm".to_string(), Box::new(hardware::HdparmCommand));
    commands.insert("cpupower".to_string(), Box::new(hardware::CpupowerCommand));
    commands.insert("powertop".to_string(), Box::new(hardware::PowertopCommand));
    commands.insert("acpi".to_string(), Box::new(hardware::AcpiCommand));
    commands.insert("battery".to_string(), Box::new(hardware::BatteryCommand));
    commands.insert("thermal".to_string(), Box::new(hardware::ThermalCommand));
    commands.insert("gpu-info".to_string(), Box::new(hardware::GpuInfoCommand));
    commands.insert("display".to_string(), Box::new(hardware::DisplayCommand));
    commands.insert("audio".to_string(), Box::new(hardware::AudioCommand));
    commands.insert("camera".to_string(), Box::new(hardware::CameraCommand));
    
    // Development commands
    commands.insert("git".to_string(), Box::new(development::GitCommand));
    commands.insert("make".to_string(), Box::new(development::MakeCommand));
    commands.insert("cmake".to_string(), Box::new(development::CmakeCommand));
    commands.insert("cargo".to_string(), Box::new(development::CargoCommand));
    commands.insert("rustc".to_string(), Box::new(development::RustcCommand));
    commands.insert("gcc".to_string(), Box::new(development::GccCommand));
    commands.insert("clang".to_string(), Box::new(development::ClangCommand));
    commands.insert("python".to_string(), Box::new(development::PythonCommand));
    commands.insert("python3".to_string(), Box::new(development::Python3Command));
    commands.insert("node".to_string(), Box::new(development::NodeCommand));
    commands.insert("npm".to_string(), Box::new(development::NpmCommand));
    commands.insert("yarn".to_string(), Box::new(development::YarnCommand));
    commands.insert("go".to_string(), Box::new(development::GoCommand));
    commands.insert("java".to_string(), Box::new(development::JavaCommand));
    commands.insert("javac".to_string(), Box::new(development::JavacCommand));
    commands.insert("gdb".to_string(), Box::new(development::GdbCommand));
    commands.insert("lldb".to_string(), Box::new(development::LldbCommand));
    commands.insert("valgrind".to_string(), Box::new(development::ValgrindCommand));
    commands.insert("strace".to_string(), Box::new(development::StraceCommand));
    commands.insert("ltrace".to_string(), Box::new(development::LtraceCommand));
    commands.insert("objdump".to_string(), Box::new(development::ObjdumpCommand));
    commands.insert("nm".to_string(), Box::new(development::NmCommand));
    commands.insert("readelf".to_string(), Box::new(development::ReadelfCommand));
    commands.insert("strings".to_string(), Box::new(development::StringsCommand));
    commands.insert("hexdump".to_string(), Box::new(development::HexdumpCommand));
    commands.insert("xxd".to_string(), Box::new(development::XxdCommand));
    commands.insert("od".to_string(), Box::new(development::OdCommand));
    
    // Multimedia commands
    commands.insert("ffmpeg".to_string(), Box::new(multimedia::FfmpegCommand));
    commands.insert("ffplay".to_string(), Box::new(multimedia::FfplayCommand));
    commands.insert("ffprobe".to_string(), Box::new(multimedia::FfprobeCommand));
    commands.insert("vlc".to_string(), Box::new(multimedia::VlcCommand));
    commands.insert("mpv".to_string(), Box::new(multimedia::MpvCommand));
    commands.insert("mplayer".to_string(), Box::new(multimedia::MplayerCommand));
    commands.insert("alsamixer".to_string(), Box::new(multimedia::AlsamixerCommand));
    commands.insert("pulseaudio".to_string(), Box::new(multimedia::PulseaudioCommand));
    commands.insert("pactl".to_string(), Box::new(multimedia::PactlCommand));
    commands.insert("aplay".to_string(), Box::new(multimedia::AplayCommand));
    commands.insert("arecord".to_string(), Box::new(multimedia::ArecordCommand));
    commands.insert("sox".to_string(), Box::new(multimedia::SoxCommand));
    commands.insert("imagemagick".to_string(), Box::new(multimedia::ImagemagickCommand));
    commands.insert("convert".to_string(), Box::new(multimedia::ConvertCommand));
    commands.insert("gimp".to_string(), Box::new(multimedia::GimpCommand));
    commands.insert("blender".to_string(), Box::new(multimedia::BlenderCommand));
    
    // Security commands
    commands.insert("openssl".to_string(), Box::new(security::OpensslCommand));
    commands.insert("gpg".to_string(), Box::new(security::GpgCommand));
    commands.insert("ssh-keygen".to_string(), Box::new(security::SshKeygenCommand));
    commands.insert("nmap".to_string(), Box::new(security::NmapCommand));
    commands.insert("wireshark".to_string(), Box::new(security::WiresharkCommand));
    commands.insert("tcpdump".to_string(), Box::new(security::TcpdumpCommand));
    commands.insert("iptables".to_string(), Box::new(security::IptablesCommand));
    commands.insert("ufw".to_string(), Box::new(security::UfwCommand));
    commands.insert("fail2ban".to_string(), Box::new(security::Fail2banCommand));
    commands.insert("chkrootkit".to_string(), Box::new(security::ChkrootkitCommand));
    commands.insert("rkhunter".to_string(), Box::new(security::RkhunterCommand));
    commands.insert("clamav".to_string(), Box::new(security::ClamavCommand));
    commands.insert("lynis".to_string(), Box::new(security::LynisCommand));
    commands.insert("aide".to_string(), Box::new(security::AideCommand));
    commands.insert("tripwire".to_string(), Box::new(security::TripwireCommand));
    
    unsafe {
        EXTERNAL_COMMANDS = Some(commands);
    }
}

/// Get external command by name
pub fn get_external_command(name: &str) -> Option<&'static dyn ExternalCommand> {
    unsafe {
        if let Some(ref commands) = EXTERNAL_COMMANDS {
            commands.get(name).map(|cmd| cmd.as_ref())
        } else {
            None
        }
    }
}

/// List all available external commands
pub fn list_external_commands() -> Vec<&'static str> {
    unsafe {
        if let Some(ref commands) = EXTERNAL_COMMANDS {
            commands.keys().map(|s| s.as_str()).collect()
        } else {
            Vec::new()
        }
    }
}

/// Get command help
pub fn get_command_help(name: &str) -> Option<(&'static str, &'static str)> {
    if let Some(cmd) = get_external_command(name) {
        Some((cmd.description(), cmd.usage()))
    } else {
        None
    }
}