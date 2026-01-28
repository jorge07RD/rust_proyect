use crate::core::history::RestoreHistoryEntry;
use crate::core::profile::Profile;
use crate::events::Action;
use tokio::sync::mpsc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Home,
    Profile,
    Backups,
    Restore,
}

impl Tab {
    pub fn next(&self) -> Self {
        match self {
            Tab::Home => Tab::Profile,
            Tab::Profile => Tab::Backups,
            Tab::Backups => Tab::Restore,
            Tab::Restore => Tab::Home,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            Tab::Home => Tab::Restore,
            Tab::Profile => Tab::Home,
            Tab::Backups => Tab::Profile,
            Tab::Restore => Tab::Backups,
        }
    }

    pub fn title(&self) -> &str {
        match self {
            Tab::Home => "Home",
            Tab::Profile => "Perfil",
            Tab::Backups => "Backups",
            Tab::Restore => "Restaurar",
        }
    }
}

#[derive(Debug, Clone)]
pub struct BackupFolder {
    pub name: String,
    pub has_dump: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProfileField {
    XHost,
    Port,
    Username,
    Password,
    ContainerId,
    DirBackup,
}

impl ProfileField {
    pub fn next(&self) -> Self {
        match self {
            ProfileField::XHost => ProfileField::Port,
            ProfileField::Port => ProfileField::Username,
            ProfileField::Username => ProfileField::Password,
            ProfileField::Password => ProfileField::ContainerId,
            ProfileField::ContainerId => ProfileField::DirBackup,
            ProfileField::DirBackup => ProfileField::XHost,
        }
    }

    pub fn previous(&self) -> Self {
        match self {
            ProfileField::XHost => ProfileField::DirBackup,
            ProfileField::Port => ProfileField::XHost,
            ProfileField::Username => ProfileField::Port,
            ProfileField::Password => ProfileField::Username,
            ProfileField::ContainerId => ProfileField::Password,
            ProfileField::DirBackup => ProfileField::ContainerId,
        }
    }
}

pub struct ProfileEditor {
    pub xhost: String,
    pub port: String,
    pub username: String,
    pub password: String,
    pub container_id: String,
    pub dir_backup: String,
    pub current_field: ProfileField,
    pub editing: bool,
}

impl ProfileEditor {
    pub fn new() -> Self {
        Self {
            xhost: String::new(),
            port: String::new(),
            username: String::new(),
            password: String::new(),
            container_id: String::new(),
            dir_backup: String::new(),
            current_field: ProfileField::XHost,
            editing: false,
        }
    }

    pub fn from_profile(profile: &Profile) -> Self {
        Self {
            xhost: profile.xhost.clone(),
            port: profile.port.to_string(),
            username: profile.username.clone(),
            password: profile.password.clone().unwrap_or_default(),
            container_id: profile.container_id.clone().unwrap_or_default(),
            dir_backup: profile.dir_backup.clone().unwrap_or_default(),
            current_field: ProfileField::XHost,
            editing: false,
        }
    }

    pub fn to_profile(&self) -> Option<Profile> {
        let port = self.port.parse::<u16>().ok()?;

        Some(Profile::new(
            self.xhost.clone(),
            port,
            self.username.clone(),
            if self.password.is_empty() { None } else { Some(self.password.clone()) },
            if self.container_id.is_empty() { None } else { Some(self.container_id.clone()) },
            if self.dir_backup.is_empty() { None } else { Some(self.dir_backup.clone()) },
        ))
    }

    pub fn get_current_value_mut(&mut self) -> &mut String {
        match self.current_field {
            ProfileField::XHost => &mut self.xhost,
            ProfileField::Port => &mut self.port,
            ProfileField::Username => &mut self.username,
            ProfileField::Password => &mut self.password,
            ProfileField::ContainerId => &mut self.container_id,
            ProfileField::DirBackup => &mut self.dir_backup,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestoreState {
    Idle,
    Restoring,
    Completed,
    Failed,
}

pub struct App {
    pub current_tab: Tab,
    pub profile: Option<Profile>,
    pub profile_editor: ProfileEditor,
    pub backups: Vec<BackupFolder>,
    pub selected_backup: Option<usize>,
    pub backup_filter: String,
    pub filtered_backups: Vec<BackupFolder>,
    pub restore_state: RestoreState,
    pub restore_logs: Vec<String>,
    pub log_scroll: u16,
    pub history: Vec<RestoreHistoryEntry>,
    pub action_tx: mpsc::UnboundedSender<Action>,
    pub action_rx: mpsc::UnboundedReceiver<Action>,
    pub is_loading: bool,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub restore_db_name: String,
    pub restore_backup_folder: Option<String>,
}

impl App {
    pub fn new() -> Self {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        Self {
            current_tab: Tab::Home,
            profile: None,
            profile_editor: ProfileEditor::new(),
            backups: Vec::new(),
            selected_backup: None,
            backup_filter: String::new(),
            filtered_backups: Vec::new(),
            restore_state: RestoreState::Idle,
            restore_logs: Vec::new(),
            log_scroll: 0,
            history: Vec::new(),
            action_tx,
            action_rx,
            is_loading: false,
            should_quit: false,
            status_message: None,
            restore_db_name: String::new(),
            restore_backup_folder: None,
        }
    }

    pub fn update_filtered_backups(&mut self) {
        if self.backup_filter.is_empty() {
            self.filtered_backups = self.backups.clone();
        } else {
            self.filtered_backups = self.backups
                .iter()
                .filter(|b| b.name.to_lowercase().contains(&self.backup_filter.to_lowercase()))
                .cloned()
                .collect();
        }
    }

    pub fn get_selected_backup(&self) -> Option<&BackupFolder> {
        self.selected_backup.and_then(|idx| self.filtered_backups.get(idx))
    }
}
