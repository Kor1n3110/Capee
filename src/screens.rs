use super::*;

impl App {
    pub(super) fn home(&mut self, ui: &mut egui::Ui, card: Color32) {
        let before = (
            self.layout,
            self.lesson,
            self.profile,
            self.selected_endless,
        );
        ui.add_space(4.);
        ui.columns(2, |cols| {
            for (i, col) in cols.iter_mut().enumerate() {
                let width = col.available_width();
                egui::Frame::new()
                    .fill(card)
                    .inner_margin(20.)
                    .corner_radius(14.)
                    .show(col, |ui| {
                        ui.set_min_width(width - 40.);
                        ui.label(
                            RichText::new(if i == 0 {
                                "ПРАКТИКА"
                            } else {
                                "ВАШ ПРОГРЕСС"
                            })
                            .small()
                            .color(MUTED),
                        );
                        ui.label(
                            RichText::new(if i == 0 {
                                "Спокойно. Точно. Каждый день.".into()
                            } else {
                                format!(
                                    "Завершено занятий: {}",
                                    self.data
                                        .records
                                        .iter()
                                        .filter(|r| r.profile == self.data.profiles[self.profile])
                                        .count()
                                )
                            })
                            .size(19.),
                        );
                    });
            }
        });
        ui.add_space(16.);
        ui.heading("01  Выберите направление");
        ui.columns(2, |cols| {
            for (column, col) in cols.iter_mut().enumerate() {
                for course in [column * 2, column * 2 + 1] {
                    let selected = self.layout == course;
                    let label = ["Русский язык", "English", "Python", "C++"][course];
                    let desc = if course < 2 {
                        "Буквы, слова и связные тексты"
                    } else {
                        "Скобки, операторы и отступы"
                    };
                    if col
                        .add_sized(
                            [col.available_width(), 66.],
                            egui::Button::new(
                                RichText::new(format!(
                                    "{}  {}\n{}",
                                    if selected { "●" } else { "○" },
                                    label,
                                    desc
                                ))
                                .size(16.)
                                .color(if selected {
                                    GREEN
                                } else if self.data.dark {
                                    Color32::LIGHT_GRAY
                                } else {
                                    GREEN
                                }),
                            )
                            .fill(if selected { LIME } else { card })
                            .corner_radius(12.),
                        )
                        .clicked()
                    {
                        self.layout = course;
                    }
                }
            }
        });
        ui.add_space(12.);
        ui.heading("02  Выберите ритм");
        ui.columns(2, |cols| {
            for (i, col) in cols.iter_mut().enumerate() {
                let endless = i == 1;
                let selected = self.selected_endless == endless;
                let text = if endless {
                    "Бесконечная практика\nНовые примеры без ограничения времени"
                } else {
                    "Урок · 100 слов\nОдна цель и результат после завершения"
                };
                if col
                    .add_sized(
                        [col.available_width(), 70.],
                        egui::Button::new(RichText::new(text).size(15.).color(if selected {
                            GREEN
                        } else if self.data.dark {
                            Color32::LIGHT_GRAY
                        } else {
                            GREEN
                        }))
                        .fill(if selected { LIME } else { card })
                        .corner_radius(12.),
                    )
                    .clicked()
                {
                    self.selected_endless = endless;
                }
            }
        });
        ui.add_space(12.);
        ui.horizontal_wrapped(|ui| {
            egui::ComboBox::from_id_salt("home_profile")
                .selected_text(&self.data.profiles[self.profile])
                .show_ui(ui, |ui| {
                    for (i, name) in self.data.profiles.iter().enumerate() {
                        ui.selectable_value(&mut self.profile, i, name);
                    }
                });
            if !self.selected_endless {
                egui::ComboBox::from_id_salt("home_lesson")
                    .selected_text(lessons::title(self.layout, self.lesson))
                    .show_ui(ui, |ui| {
                        for i in 0..8 {
                            ui.selectable_value(
                                &mut self.lesson,
                                i,
                                lessons::title(self.layout, i),
                            );
                        }
                    });
            } else {
                ui.label(
                    RichText::new("Примеры выбранного направления сменяются автоматически")
                        .small()
                        .color(MUTED),
                );
            }
        });
        if before
            != (
                self.layout,
                self.lesson,
                self.profile,
                self.selected_endless,
            )
        {
            self.reset();
        }
        ui.add_space(16.);
        ui.horizontal(|ui| {
            if ui
                .add_sized(
                    [250., 48.],
                    egui::Button::new(
                        RichText::new("Начать заниматься →")
                            .color(GREEN)
                            .strong()
                            .size(17.),
                    )
                    .fill(LIME)
                    .corner_radius(10.),
                )
                .clicked()
            {
                self.reset();
                self.page = 0;
            }
            if self.run.attempts > 0 && !self.run.done && ui.button("Продолжить занятие").clicked()
            {
                if self.run.paused {
                    self.run.pause();
                }
                self.page = 0;
            }
        });
        ui.add_space(8.);
        ui.label(RichText::new("Первые нажатия запустят таймер. Держите кисти расслабленными — точность важнее скорости.").small().color(MUTED));
    }

    pub(super) fn training(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        card: Color32,
        dark: bool,
        compact: bool,
    ) {
        ui.horizontal(|ui| {
            if ui.button("← В меню").clicked() {
                if self.run.start.is_some() {
                    self.run.pause();
                }
                self.page = 4;
            }
            ui.label(RichText::new(COURSES[self.layout]).color(MUTED));
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("Начать заново").clicked() {
                    self.reset();
                }
            });
        });
        ui.add_space(4.);
        ui.columns(3, |cols| {
            for (col, (label, value)) in cols.iter_mut().zip([
                ("Скорость", format!("{:.0} зн/мин", self.run.speed())),
                ("Точность", format!("{:.0}%", self.run.accuracy())),
                (
                    "Время",
                    format!(
                        "{:02}:{:02}",
                        self.run.seconds() as u64 / 60,
                        self.run.seconds() as u64 % 60
                    ),
                ),
            ]) {
                let width = col.available_width();
                egui::Frame::new()
                    .fill(card)
                    .inner_margin(12.)
                    .corner_radius(12.)
                    .show(col, |ui| {
                        ui.set_min_width(width - 24.);
                        ui.label(RichText::new(label).small().color(MUTED));
                        ui.label(RichText::new(value).size(26.).strong());
                    });
            }
        });
        let width = ui.available_width();
        egui::Frame::new()
            .fill(card)
            .inner_margin(18.)
            .corner_radius(16.)
            .show(ui, |ui| {
                ui.set_min_width(width - 36.);
                ui.horizontal(|ui| {
                    ui.label(
                        RichText::new(if self.run.endless {
                            "БЕСКОНЕЧНАЯ ПРАКТИКА".into()
                        } else {
                            format!(
                                "УРОК {} / 8 · {}",
                                self.lesson + 1,
                                lessons::title(self.layout, self.lesson)
                            )
                        })
                        .small()
                        .color(MUTED),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            RichText::new(if self.run.done {
                                "Завершено"
                            } else if self.run.paused {
                                "Пауза"
                            } else {
                                "Курсор в центре"
                            })
                            .small()
                            .color(MUTED),
                        );
                    });
                });
                self.moving_text(ui, dark, compact);
            });
        // Put the essential controls before the keyboard so they remain reachable on shorter screens.
        ui.horizontal_wrapped(|ui| {
            if !self.run.done
                && ui
                    .button(if self.run.paused {
                        "Продолжить"
                    } else {
                        "Пауза"
                    })
                    .clicked()
            {
                self.run.pause();
            }
            if self.run.endless && !self.run.done {
                if ui
                    .add_enabled(
                        self.run.attempts > 0,
                        egui::Button::new("Завершить и сохранить"),
                    )
                    .clicked()
                {
                    self.run.finish();
                    self.record_result();
                }
            } else if self.run.done && !self.run.endless && ui.button("Следующий урок →").clicked()
            {
                self.lesson = (self.lesson + 1).min(7);
                self.reset();
            }
            ui.label(
                RichText::new(if self.run.endless {
                    format!(
                        "{} символов · ошибок: {}",
                        self.run.typed(),
                        self.run.errors
                    )
                } else {
                    format!(
                        "{} / {} символов · ошибок: {}",
                        self.run.typed(),
                        self.run.text.len(),
                        self.run.errors
                    )
                })
                .color(MUTED),
            );
            if self.run.done {
                ui.label(
                    RichText::new(if self.notice.is_empty() {
                        "Результат сохранён"
                    } else {
                        "Проверьте ошибку сохранения выше"
                    })
                    .color(if dark { LIME } else { GREEN }),
                );
            }
        });
        if !self.run.endless {
            ui.add(
                egui::ProgressBar::new(
                    self.run.position as f32 / self.run.text.len().max(1) as f32,
                )
                .fill(if dark { LIME } else { GREEN }),
            );
        }
        let width = ui.available_width();
        egui::Frame::new()
            .fill(card)
            .inner_margin(16.)
            .corner_radius(16.)
            .show(ui, |ui| {
                ui.set_min_width(width - 32.);
                self.keyboard(ui);
            });
        if self.page == 0 && !ctx.wants_keyboard_input() {
            let events = ctx.input(|i| i.events.clone());
            for event in events {
                let text = match event {
                    egui::Event::Text(t) => t,
                    egui::Event::Key {
                        key: egui::Key::Enter,
                        pressed: true,
                        ..
                    } => "\n".into(),
                    egui::Event::Key {
                        key: egui::Key::Tab,
                        pressed: true,
                        ..
                    } => "    ".into(),
                    _ => String::new(),
                };
                for c in text.chars() {
                    self.run.input(c);
                    if self.run.wrong && self.data.sound {
                        let _ = std::process::Command::new("canberra-gtk-play")
                            .args(["-i", "dialog-error"])
                            .spawn();
                    }
                    self.record_result();
                }
            }
        }
    }
}
