use crate::app::{App, BackupFolder, RestoreState, Tab};
use crate::core::database::DatabaseManager;
use crate::core::history::{HistoryManager, RestoreHistoryEntry};
use crate::core::profile;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug, Clone)]
pub enum Action {
    // Navigation
    NextTab,
    PreviousTab,
    SwitchToTab(Tab),
    Quit,

    // Profile
    EditProfile,
    SaveProfile,
    CancelProfileEdit,
    ProfileFieldNext,
    ProfileFieldPrevious,
    ProfileFieldStartEdit,
    ProfileFieldStopEdit,
    ProfileFieldInput(char),
    ProfileFieldBackspace,

    // Backups
    RefreshBackups,
    SelectBackupNext,
    SelectBackupPrevious,
    FilterBackupsInput(char),
    FilterBackupsBackspace,
    FilterBackupsClear,
    BackupsLoaded(Vec<BackupFolder>),

    // Restore
    StartRestore,
    CancelRestore,
    ScrollLogsUp,
    ScrollLogsDown,
    RestoreProgress(String),
    RestoreCompleted(bool, Option<String>),
    RestoreDbNameInput(char),
    RestoreDbNameBackspace,

    // General
    Error(String),
}

/// Check if the user is currently in an input mode where number key navigation should be disabled
fn is_in_number_input_mode(app: &App) -> bool {
    match app.current_tab {
        Tab::Profile => app.profile_editor.editing,
        Tab::Backups => !app.backup_filter.is_empty(),
        // In Restore, disable number navigation only if user has started typing
        Tab::Restore => app.restore_state == RestoreState::Idle && !app.restore_db_name.is_empty(),
        _ => false,
    }
}

/// Check if the user is in a mode where Tab key navigation should be disabled
fn is_in_editing_mode(app: &App) -> bool {
    match app.current_tab {
        Tab::Profile => app.profile_editor.editing,
        Tab::Backups => !app.backup_filter.is_empty(),
        // In Restore, allow Tab navigation even when typing
        _ => false,
    }
}

pub fn handle_key_event(key: KeyEvent, app: &App) -> Option<Action> {
    // Global shortcuts
    if key.modifiers.contains(KeyModifiers::CONTROL) {
        match key.code {
            KeyCode::Char('c') => return Some(Action::Quit),
            KeyCode::Char('s') if app.current_tab == Tab::Profile => return Some(Action::SaveProfile),
            _ => {}
        }
    }

    // Check if user is in input modes
    let in_number_input = is_in_number_input_mode(app);
    let in_editing = is_in_editing_mode(app);

    match key.code {
        KeyCode::Char('q') if !in_editing && app.restore_state == RestoreState::Idle => {
            return Some(Action::Quit)
        }
        // Tab navigation: disabled only when actively editing fields (Profile or Backups filter)
        KeyCode::Tab if !in_editing => return Some(Action::NextTab),
        KeyCode::BackTab if !in_editing => return Some(Action::PreviousTab),
        // Number navigation: disabled when typing in any input field
        KeyCode::Char('1') if !in_number_input => return Some(Action::SwitchToTab(Tab::Home)),
        KeyCode::Char('2') if !in_number_input => return Some(Action::SwitchToTab(Tab::Profile)),
        KeyCode::Char('3') if !in_number_input => return Some(Action::SwitchToTab(Tab::Backups)),
        KeyCode::Char('4') if !in_number_input => return Some(Action::SwitchToTab(Tab::Restore)),
        _ => {}
    }

    // Tab-specific handling
    match app.current_tab {
        Tab::Home => handle_home_keys(key, app),
        Tab::Profile => handle_profile_keys(key, app),
        Tab::Backups => handle_backups_keys(key, app),
        Tab::Restore => handle_restore_keys(key, app),
    }
}

fn handle_home_keys(_key: KeyEvent, _app: &App) -> Option<Action> {
    None
}

fn handle_profile_keys(key: KeyEvent, app: &App) -> Option<Action> {
    if app.profile_editor.editing {
        match key.code {
            KeyCode::Esc => return Some(Action::ProfileFieldStopEdit),
            KeyCode::Enter => return Some(Action::ProfileFieldStopEdit),
            KeyCode::Char(c) => return Some(Action::ProfileFieldInput(c)),
            KeyCode::Backspace => return Some(Action::ProfileFieldBackspace),
            _ => {}
        }
    } else {
        match key.code {
            KeyCode::Char('e') => return Some(Action::EditProfile),
            KeyCode::Up => return Some(Action::ProfileFieldPrevious),
            KeyCode::Down => return Some(Action::ProfileFieldNext),
            KeyCode::Enter => return Some(Action::ProfileFieldStartEdit),
            KeyCode::Esc => return Some(Action::CancelProfileEdit),
            _ => {}
        }
    }
    None
}

fn handle_backups_keys(key: KeyEvent, app: &App) -> Option<Action> {
    match key.code {
        KeyCode::Char('r') if app.backup_filter.is_empty() => return Some(Action::RefreshBackups),
        KeyCode::Up => return Some(Action::SelectBackupPrevious),
        KeyCode::Down => return Some(Action::SelectBackupNext),
        KeyCode::Char('/') if app.backup_filter.is_empty() => return Some(Action::FilterBackupsInput('/')),
        KeyCode::Char(c) if !app.backup_filter.is_empty() => return Some(Action::FilterBackupsInput(c)),
        KeyCode::Backspace if !app.backup_filter.is_empty() => return Some(Action::FilterBackupsBackspace),
        KeyCode::Esc if !app.backup_filter.is_empty() => return Some(Action::FilterBackupsClear),
        KeyCode::Enter if app.get_selected_backup().is_some() => {
            return Some(Action::SwitchToTab(Tab::Restore));
        }
        _ => {}
    }
    None
}

fn handle_restore_keys(key: KeyEvent, app: &App) -> Option<Action> {
    match app.restore_state {
        RestoreState::Idle => {
            match key.code {
                KeyCode::Enter if !app.restore_db_name.is_empty() => {
                    return Some(Action::StartRestore);
                }
                KeyCode::Char(c) => return Some(Action::RestoreDbNameInput(c)),
                KeyCode::Backspace => return Some(Action::RestoreDbNameBackspace),
                _ => {}
            }
        }
        RestoreState::Restoring => {
            match key.code {
                KeyCode::Esc => return Some(Action::CancelRestore),
                KeyCode::Up => return Some(Action::ScrollLogsUp),
                KeyCode::Down => return Some(Action::ScrollLogsDown),
                _ => {}
            }
        }
        RestoreState::Completed | RestoreState::Failed => {
            match key.code {
                KeyCode::Enter | KeyCode::Esc => {
                    // Reset to idle state
                    return None; // Handle in update
                }
                KeyCode::Up => return Some(Action::ScrollLogsUp),
                KeyCode::Down => return Some(Action::ScrollLogsDown),
                _ => {}
            }
        }
    }
    None
}

impl App {
    pub async fn update(&mut self, action: Action) -> anyhow::Result<()> {
        match action {
            Action::Quit => {
                self.should_quit = true;
            }
            Action::NextTab => {
                self.current_tab = self.current_tab.next();
            }
            Action::PreviousTab => {
                self.current_tab = self.current_tab.previous();
            }
            Action::SwitchToTab(tab) => {
                self.current_tab = tab;
                // If switching to Restore, preload selected backup
                if tab == Tab::Restore {
                    if let Some(backup) = self.get_selected_backup() {
                        self.restore_backup_folder = Some(backup.name.clone());
                    }
                }
            }
            Action::EditProfile => {
                if let Some(ref profile) = self.profile {
                    self.profile_editor = crate::app::ProfileEditor::from_profile(profile);
                } else {
                    self.profile_editor = crate::app::ProfileEditor::new();
                }
            }
            Action::SaveProfile => {
                if let Some(profile) = self.profile_editor.to_profile() {
                    match profile::save_profile(&profile) {
                        Ok(_) => {
                            self.profile = Some(profile);
                            self.status_message = Some("Perfil guardado exitosamente".to_string());
                        }
                        Err(e) => {
                            self.status_message = Some(format!("Error al guardar perfil: {}", e));
                        }
                    }
                } else {
                    self.status_message = Some("Datos de perfil inválidos".to_string());
                }
            }
            Action::CancelProfileEdit => {
                if let Some(ref profile) = self.profile {
                    self.profile_editor = crate::app::ProfileEditor::from_profile(profile);
                }
            }
            Action::ProfileFieldNext => {
                self.profile_editor.current_field = self.profile_editor.current_field.next();
            }
            Action::ProfileFieldPrevious => {
                self.profile_editor.current_field = self.profile_editor.current_field.previous();
            }
            Action::ProfileFieldStartEdit => {
                self.profile_editor.editing = true;
            }
            Action::ProfileFieldStopEdit => {
                self.profile_editor.editing = false;
            }
            Action::ProfileFieldInput(c) => {
                self.profile_editor.get_current_value_mut().push(c);
            }
            Action::ProfileFieldBackspace => {
                self.profile_editor.get_current_value_mut().pop();
            }
            Action::RefreshBackups => {
                self.is_loading = true;
                let tx = self.action_tx.clone();
                if let Some(ref profile) = self.profile {
                    let profile = profile.clone();
                    tokio::spawn(async move {
                        match load_backups(&profile).await {
                            Ok(backups) => {
                                let _ = tx.send(Action::BackupsLoaded(backups));
                            }
                            Err(e) => {
                                let _ = tx.send(Action::Error(format!("Error loading backups: {}", e)));
                            }
                        }
                    });
                }
            }
            Action::BackupsLoaded(backups) => {
                self.backups = backups;
                self.update_filtered_backups();
                self.is_loading = false;
                if !self.filtered_backups.is_empty() && self.selected_backup.is_none() {
                    self.selected_backup = Some(0);
                }
            }
            Action::SelectBackupNext => {
                if !self.filtered_backups.is_empty() {
                    self.selected_backup = Some(
                        self.selected_backup
                            .map(|i| (i + 1) % self.filtered_backups.len())
                            .unwrap_or(0),
                    );
                }
            }
            Action::SelectBackupPrevious => {
                if !self.filtered_backups.is_empty() {
                    self.selected_backup = Some(
                        self.selected_backup
                            .map(|i| {
                                if i == 0 {
                                    self.filtered_backups.len() - 1
                                } else {
                                    i - 1
                                }
                            })
                            .unwrap_or(0),
                    );
                }
            }
            Action::FilterBackupsInput(c) => {
                self.backup_filter.push(c);
                self.update_filtered_backups();
                self.selected_backup = if !self.filtered_backups.is_empty() {
                    Some(0)
                } else {
                    None
                };
            }
            Action::FilterBackupsBackspace => {
                self.backup_filter.pop();
                self.update_filtered_backups();
            }
            Action::FilterBackupsClear => {
                self.backup_filter.clear();
                self.update_filtered_backups();
            }
            Action::StartRestore => {
                if let Some(ref profile) = self.profile {
                    if let Some(ref backup_folder) = self.restore_backup_folder {
                        self.restore_state = RestoreState::Restoring;
                        self.restore_logs.clear();
                        self.log_scroll = 0;

                        let profile = profile.clone();
                        let db_name = self.restore_db_name.clone();
                        let backup_folder = backup_folder.clone();
                        let tx = self.action_tx.clone();

                        tokio::spawn(async move {
                            let db_manager = DatabaseManager::new(profile);

                            let result = db_manager
                                .restore_database(&db_name, &backup_folder, |log| {
                                    let _ = tx.send(Action::RestoreProgress(log));
                                })
                                .await;

                            match result {
                                Ok(_) => {
                                    let _ = tx.send(Action::RestoreCompleted(true, None));
                                }
                                Err(e) => {
                                    let _ = tx.send(Action::RestoreCompleted(false, Some(e.to_string())));
                                }
                            }
                        });
                    }
                }
            }
            Action::CancelRestore => {
                self.restore_state = RestoreState::Idle;
            }
            Action::RestoreProgress(log) => {
                self.restore_logs.push(log);
                // Auto-scroll to the bottom to show the latest log
                // We set a large value and the UI will clamp it to the maximum
                self.log_scroll = u16::MAX;
            }
            Action::RestoreCompleted(success, error) => {
                self.restore_state = if success {
                    RestoreState::Completed
                } else {
                    RestoreState::Failed
                };

                if let Some(ref err) = error {
                    self.restore_logs.push(format!("ERROR: {}", err));
                }

                // Add to history
                let mut history = HistoryManager::load().unwrap_or_else(|_| HistoryManager::new());
                let entry = RestoreHistoryEntry::new(
                    self.restore_db_name.clone(),
                    self.restore_backup_folder.clone().unwrap_or_default(),
                    success,
                    error,
                );
                let _ = history.add_entry(entry);
                self.history = history.get_recent(10);
            }
            Action::RestoreDbNameInput(c) => {
                self.restore_db_name.push(c);
            }
            Action::RestoreDbNameBackspace => {
                self.restore_db_name.pop();
            }
            Action::ScrollLogsUp => {
                self.log_scroll = self.log_scroll.saturating_sub(1);
            }
            Action::ScrollLogsDown => {
                self.log_scroll = self.log_scroll.saturating_add(1);
            }
            Action::Error(msg) => {
                self.status_message = Some(msg);
                self.is_loading = false;
            }
        }
        Ok(())
    }
}

async fn load_backups(profile: &crate::core::profile::Profile) -> anyhow::Result<Vec<BackupFolder>> {
    let db_manager = DatabaseManager::new(profile.clone());
    let base_dir = profile.dir_backup.as_ref()
        .map(|s| s.as_str())
        .unwrap_or("/tmp/backups");

    let dirs = db_manager.executor().list_directories(base_dir).await?;

    let mut backups = Vec::new();
    for dir in dirs {
        let dump_path = format!("{}/{}/dump.sql", base_dir, dir);
        let has_dump = db_manager.executor().file_exists(&dump_path).await?;
        backups.push(BackupFolder {
            name: dir,
            has_dump,
        });
    }

    Ok(backups)
}
