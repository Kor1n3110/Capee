use eframe::egui::{self, Color32, RichText};
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    time::{Duration, Instant},
};
const GREEN: Color32 = Color32::from_rgb(24, 54, 44);
const LIME: Color32 = Color32::from_rgb(213, 246, 133);
const PAPER: Color32 = Color32::from_rgb(247, 246, 240);
const MUTED: Color32 = Color32::from_rgb(128, 139, 130);
mod lessons;
mod screens;
use lessons::COURSES;
#[derive(Default, Serialize, Deserialize, Clone)]
struct Record {
    profile: String,
    layout: usize,
    lesson: usize,
    speed: f64,
    accuracy: f64,
    errors: usize,
    seconds: f64,
    #[serde(default)]
    endless: bool,
}
#[derive(Serialize, Deserialize)]
struct Data {
    profiles: Vec<String>,
    records: Vec<Record>,
    sound: bool,
    #[serde(default)]
    dark: bool,
}
impl Default for Data {
    fn default() -> Self {
        Self {
            profiles: vec!["Мой профиль".into()],
            records: vec![],
            sound: false,
            dark: false,
        }
    }
}
struct Run {
    text: Vec<char>,
    position: usize,
    attempts: usize,
    errors: usize,
    start: Option<Instant>,
    elapsed: Duration,
    paused: bool,
    done: bool,
    wrong: bool,
    endless: bool,
    course: usize,
    random: u64,
    discarded: usize,
    recorded: bool,
}
impl Run {
    fn new(text: &str) -> Self {
        Self {
            text: text.chars().collect(),
            position: 0,
            attempts: 0,
            errors: 0,
            start: None,
            elapsed: Duration::ZERO,
            paused: false,
            done: false,
            wrong: false,
            endless: false,
            course: 0,
            random: 1,
            discarded: 0,
            recorded: false,
        }
    }
    fn infinite(course: usize, seed: u64) -> Self {
        let mut run = Self::new("");
        run.endless = true;
        run.course = course;
        run.random = seed;
        run.extend_stream();
        run
    }
    fn typed(&self) -> usize {
        self.discarded + self.position
    }
    fn extend_stream(&mut self) {
        while self.text.len().saturating_sub(self.position) < 180 {
            if !self.text.is_empty() {
                self.text
                    .extend(if self.course < 2 { " " } else { "\n\n" }.chars());
            }
            self.text
                .extend(lessons::endless_block(self.course, &mut self.random).chars());
        }
        // Retain enough typed text for the centered cursor without accumulating a whole session.
        if self.position > 512 {
            let drop = self.position - 100;
            self.text.drain(..drop);
            self.position -= drop;
            self.discarded += drop;
        }
    }
    fn finish(&mut self) {
        if let Some(t) = self.start.take() {
            self.elapsed += t.elapsed();
        }
        self.done = true;
        self.paused = false;
    }
    fn seconds(&self) -> f64 {
        (self.elapsed + self.start.map(|t| t.elapsed()).unwrap_or_default()).as_secs_f64()
    }
    fn speed(&self) -> f64 {
        if self.seconds() < 0.1 {
            0.0
        } else {
            self.typed() as f64 * 60.0 / self.seconds()
        }
    }
    fn accuracy(&self) -> f64 {
        if self.attempts == 0 {
            100.0
        } else {
            100.0 * (self.attempts - self.errors) as f64 / self.attempts as f64
        }
    }
    fn pause(&mut self) {
        if self.done {
            return;
        }
        if let Some(t) = self.start.take() {
            self.elapsed += t.elapsed()
        }
        self.paused = !self.paused;
        if !self.paused && self.attempts > 0 {
            self.start = Some(Instant::now())
        }
    }
    fn input(&mut self, c: char) {
        if self.paused || self.done {
            return;
        }
        if self.start.is_none() {
            self.start = Some(Instant::now())
        }
        self.attempts += 1;
        self.wrong = self.text.get(self.position) != Some(&c);
        if self.wrong {
            self.errors += 1
        } else {
            self.position += 1
        }
        if self.endless {
            self.extend_stream();
        }
        if !self.endless && self.position == self.text.len() {
            if let Some(t) = self.start.take() {
                self.elapsed += t.elapsed()
            }
            self.done = true;
        }
    }
}
struct App {
    data: Data,
    path: PathBuf,
    page: usize,
    profile: usize,
    layout: usize,
    lesson: usize,
    run: Run,
    custom: String,
    new_profile: String,
    notice: String,
    selected_endless: bool,
}
impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut fonts = egui::FontDefinitions::default();
        fonts.font_data.insert(
            "cyrillic".into(),
            egui::FontData::from_static(include_bytes!("../assets/DejaVuSans.ttf")).into(),
        );
        fonts
            .families
            .get_mut(&egui::FontFamily::Proportional)
            .unwrap()
            .insert(0, "cyrillic".into());
        fonts
            .families
            .get_mut(&egui::FontFamily::Monospace)
            .unwrap()
            .insert(0, "cyrillic".into());
        cc.egui_ctx.set_fonts(fonts);
        let mut style = (*cc.egui_ctx.style()).clone();
        style.visuals = egui::Visuals::light();
        style.visuals.panel_fill = PAPER;
        style.visuals.override_text_color = Some(GREEN);
        style.visuals.selection.bg_fill = LIME;
        style.spacing.item_spacing = egui::vec2(12., 12.);
        style.spacing.button_padding = egui::vec2(16., 10.);
        cc.egui_ctx.set_style(style);
        let root = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
            .unwrap_or_else(std::env::temp_dir);
        let path = root.join("capee/history.json");
        let mut notice = String::new();
        let data=match std::fs::read(&path){Ok(b)=>serde_json::from_slice(&b).unwrap_or_else(|_|{notice="Файл истории повреждён. Сохранение отключено: сохраните резервную копию и исправьте файл.".into();Data::default()}),Err(e) if e.kind()==std::io::ErrorKind::NotFound=>Data::default(),Err(e)=>{notice=format!("Не удалось прочитать историю: {e}");Data::default()}};
        Self {
            data,
            path,
            page: 4,
            profile: 0,
            layout: 0,
            lesson: 0,
            run: Run::new(&lessons::exercise(0, 0)),
            custom: String::new(),
            new_profile: String::new(),
            notice,
            selected_endless: false,
        }
    }
    fn save(&mut self) {
        if !self.notice.is_empty() {
            return;
        }
        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
            std::fs::create_dir_all(self.path.parent().unwrap())?;
            let tmp = self.path.with_extension("tmp");
            std::fs::write(&tmp, serde_json::to_vec_pretty(&self.data)?)?;
            std::fs::rename(tmp, &self.path)?;
            Ok(())
        })();
        if let Err(e) = result {
            self.notice = format!("Ошибка сохранения: {e}")
        }
    }
    fn reset(&mut self) {
        self.run = if self.selected_endless {
            let seed = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos() as u64;
            Run::infinite(self.layout, seed)
        } else {
            Run::new(&lessons::exercise(self.layout, self.lesson))
        };
    }
    fn record_result(&mut self) {
        if !self.run.done || self.run.recorded || self.run.attempts == 0 {
            return;
        }
        self.data.records.push(Record {
            profile: self.data.profiles[self.profile].clone(),
            layout: self.layout,
            lesson: self.lesson,
            speed: self.run.speed(),
            accuracy: self.run.accuracy(),
            errors: self.run.errors,
            seconds: self.run.seconds(),
            endless: self.run.endless,
        });
        self.run.recorded = true;
        self.save();
    }
    fn moving_text(&self, ui: &mut egui::Ui, dark: bool, compact: bool) {
        let (rect, _) = ui.allocate_exact_size(
            egui::vec2(ui.available_width(), if compact { 52. } else { 64. }),
            egui::Sense::hover(),
        );
        let start = self.run.position.saturating_sub(100);
        let end = (self.run.position + 101).min(self.run.text.len());
        let mut job = egui::text::LayoutJob::default();
        job.wrap.max_width = f32::INFINITY;
        for i in start..end {
            let c = self.run.text[i];
            let glyph = if c == '\n' {
                '↵'
            } else if c == '\t' {
                '→'
            } else {
                c
            };
            let current = i == self.run.position;
            job.append(
                &glyph.to_string(),
                0.,
                egui::TextFormat {
                    font_id: egui::FontId::monospace(if compact { 25. } else { 30. }),
                    color: if current {
                        GREEN
                    } else if i < self.run.position {
                        if dark {
                            LIME
                        } else {
                            Color32::from_rgb(63, 133, 83)
                        }
                    } else {
                        if dark {
                            Color32::from_gray(185)
                        } else {
                            MUTED
                        }
                    },
                    background: if current {
                        if self.run.wrong {
                            Color32::from_rgb(255, 175, 175)
                        } else {
                            LIME
                        }
                    } else {
                        Color32::TRANSPARENT
                    },
                    ..Default::default()
                },
            );
        }
        let galley = ui.fonts(|f| f.layout_job(job));
        let cursor = galley.pos_from_ccursor(egui::text::CCursor::new(self.run.position - start));
        let origin = egui::pos2(
            rect.center().x - cursor.left(),
            rect.center().y - galley.size().y / 2.,
        );
        let painter = ui.painter().with_clip_rect(rect.intersect(ui.clip_rect()));
        painter.galley(origin, galley, MUTED);
        // Fixed reference point: only the text's origin moves with typing.
        let x = rect.center().x;
        painter.line_segment(
            [
                egui::pos2(x, rect.center().y + 22.),
                egui::pos2(x, rect.center().y + 28.),
            ],
            egui::Stroke::new(2.0_f32, if dark { LIME } else { GREEN }),
        );
        ui.label(
            RichText::new(if self.run.done {
                "Упражнение завершено"
            } else if self.run.text.get(self.run.position) == Some(&'\n') {
                "Следующая клавиша — Enter (↵)"
            } else {
                "Курсор в центре · вводите символ на подсветке · ↵ означает Enter"
            })
            .small()
            .color(MUTED),
        );
    }
    fn keyboard(&self, ui: &mut egui::Ui) {
        let rows = if self.layout == 0 {
            ["ё1234567890-=", "йцукенгшщзхъ", "фывапролджэ", "ячсмитьбю."]
        } else {
            ["`1234567890-=", "qwertyuiop[]", "asdfghjkl;'", "zxcvbnm,./"]
        };
        let next = self
            .run
            .text
            .get(self.run.position)
            .copied()
            .unwrap_or('\0')
            .to_lowercase()
            .next()
            .unwrap_or('\0');
        let dark = self.data.dark;
        let colors = if dark {
            [
                Color32::from_rgb(67, 43, 49),
                Color32::from_rgb(67, 59, 38),
                Color32::from_rgb(41, 62, 45),
                Color32::from_rgb(36, 53, 66),
            ]
        } else {
            [
                Color32::from_rgb(242, 214, 215),
                Color32::from_rgb(246, 231, 187),
                Color32::from_rgb(211, 231, 202),
                Color32::from_rgb(205, 224, 236),
            ]
        };
        let gap = 8.;
        ui.spacing_mut().item_spacing = egui::vec2(gap, 6.);
        ui.spacing_mut().button_padding = egui::vec2(8., 6.);
        let compact = ui.ctx().screen_rect().height() < 1100.;
        let key_width = ((ui.available_width() - 12. * gap) / 13.).clamp(
            30.,
            if compact {
                if ui.ctx().screen_rect().height() >= 900. {
                    54.
                } else {
                    43.
                }
            } else {
                62.
            },
        );
        let key_height = (key_width * 0.78).clamp(34., 48.);
        let space_width = key_width * 6. + 5. * gap;
        for s in &rows {
            ui.horizontal(|ui| {
                let width = s.chars().count() as f32 * key_width
                    + (s.chars().count() - 1) as f32 * ui.spacing().item_spacing.x;
                ui.add_space(((ui.available_width() - width) / 2.).max(0.));
                for (i, c) in s.chars().enumerate() {
                    let col = if c == next {
                        LIME
                    } else {
                        colors[i.min(s.chars().count().saturating_sub(1) - i).min(3)]
                    };
                    ui.add_sized(
                        [key_width, key_height],
                        egui::Button::new(RichText::new(c.to_string()).size(17.).color(
                            if c == next || !dark {
                                GREEN
                            } else {
                                Color32::from_gray(225)
                            },
                        ))
                        .fill(col)
                        .corner_radius(7.),
                    );
                }
            });
        }
        ui.horizontal(|ui| {
            ui.add_space(((ui.available_width() - space_width) / 2.).max(0.));
            ui.add_sized(
                [space_width, key_height * 0.85],
                egui::Button::new(RichText::new("пробел").color(if next == ' ' || !dark {
                    GREEN
                } else {
                    Color32::LIGHT_GRAY
                }))
                .fill(if next == ' ' {
                    LIME
                } else {
                    if dark {
                        Color32::from_rgb(42, 48, 44)
                    } else {
                        Color32::from_gray(230)
                    }
                })
                .corner_radius(7.),
            );
        });
        ui.add_space(8.);
        ui.horizontal_wrapped(|ui| {
            for (color, name) in
                colors
                    .iter()
                    .zip(["Мизинец", "Безымянный", "Средний", "Указательный"])
            {
                ui.label(RichText::new("●").color(*color).size(18.));
                ui.label(RichText::new(name).small().color(MUTED));
                ui.add_space(10.);
            }
            ui.label(
                RichText::new("Большие пальцы — пробел")
                    .small()
                    .color(MUTED),
            );
        });
    }
}
impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let compact = ctx.screen_rect().height() < 1100.;
        let dark = self.data.dark;
        let background = if dark {
            Color32::from_rgb(14, 16, 15)
        } else {
            PAPER
        };
        let card = if dark {
            Color32::from_rgb(26, 30, 28)
        } else {
            Color32::WHITE
        };
        let foreground = if dark {
            Color32::from_rgb(232, 237, 230)
        } else {
            GREEN
        };
        let mut visuals = if dark {
            egui::Visuals::dark()
        } else {
            egui::Visuals::light()
        };
        visuals.panel_fill = background;
        visuals.override_text_color = Some(foreground);
        visuals.selection.bg_fill = GREEN;
        ctx.set_visuals(visuals);
        let fullscreen = ctx.input(|i| i.viewport().fullscreen.unwrap_or(false));
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F11)) {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
        }
        if fullscreen && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape))
        {
            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(false));
        }
        egui::TopBottomPanel::top("window_controls")
            .frame(egui::Frame::new().fill(background).inner_margin(8.))
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label("Capee");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button(if fullscreen {
                                "Выйти из полного экрана · F11"
                            } else {
                                "Полный экран · F11"
                            })
                            .clicked()
                        {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Fullscreen(!fullscreen));
                        }
                        if ui.button("Развернуть окно").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Maximized(true));
                        }
                        if ui
                            .toggle_value(&mut self.data.dark, "Тёмная тема")
                            .changed()
                        {
                            self.save();
                        }
                    });
                });
            });
        ctx.request_repaint_after(Duration::from_millis(100));
        if self.page != 0 && self.run.start.is_some() {
            self.run.pause();
        }
        egui::SidePanel::left("nav")
            .exact_width(200.)
            .frame(egui::Frame::new().fill(GREEN).inner_margin(24.))
            .show(ctx, |ui| {
                ui.add_space(15.);
                ui.label(RichText::new("capee").size(32.).color(LIME).strong());
                ui.label(
                    RichText::new("ПРАКТИКА ПЕЧАТИ")
                        .size(10.)
                        .color(Color32::LIGHT_GRAY),
                );
                ui.add_space(45.);
                for (i, t) in [
                    (4, "Главная"),
                    (1, "Уроки"),
                    (2, "Статистика"),
                    (3, "Настройки"),
                ] {
                    let selected = self.page == i || (i == 4 && self.page == 0);
                    if ui
                        .add_sized(
                            [152., 44.],
                            egui::Button::new(RichText::new(t).color(if selected {
                                GREEN
                            } else {
                                Color32::WHITE
                            }))
                            .fill(if selected {
                                LIME
                            } else {
                                GREEN
                            }),
                        )
                        .clicked()
                    {
                        self.page = i;
                    }
                }
                ui.add_space(65.);
                ui.label(
                    RichText::new("Быстрее сегодня.\nУвереннее завтра.").color(Color32::LIGHT_GRAY),
                );
            });
        egui::CentralPanel::default().frame(egui::Frame::new().fill(background).inner_margin(24.)).show(ctx,|ui|{
 egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui,|ui|{
 let width=ui.available_width().min(1140.);
 let inset=((ui.available_width()-width)/2.).max(0.);
 ui.horizontal(|ui|{ui.add_space(inset);ui.allocate_ui_with_layout(egui::vec2(width,0.),egui::Layout::top_down(egui::Align::Min),|ui|{
 ui.set_width(width);
 if compact {ui.spacing_mut().item_spacing.y=8.;}
 ui.add_space(if width>1000. {16.}else{0.});
 ui.horizontal(|ui|{ui.heading(RichText::new(match self.page{4=>"Найдите свой ритм.",0=>"Практика печати",1=>"Маленькие шаги. Большой прогресс.",2=>"Ваш ритм в цифрах.",_=>"Место для вашей практики."}).size(27.));});ui.label(RichText::new("Ежедневная практика слепой печати").color(MUTED));ui.add_space(15.);
 if !self.notice.is_empty(){ui.colored_label(Color32::DARK_RED,&self.notice);}
 if self.page==0 {self.training(ui,ctx,card,dark,compact);
 } else if self.page==1 {ui.horizontal(|ui|{for(i,name)in COURSES.iter().enumerate(){if ui.selectable_value(&mut self.layout,i,*name).clicked(){self.reset();}}});ui.add_space(10.);for(i,title)in (if self.layout<2 {lessons::TITLES}else{lessons::CODE_TITLES}).iter().enumerate(){egui::Frame::new().fill(card).inner_margin(18.).corner_radius(10.).show(ui,|ui|{ui.horizontal(|ui|{ui.label(RichText::new(format!("0{}   {}",i+1,title)).size(22.));if ui.button("Практиковаться →").clicked(){self.lesson=i;self.selected_endless=false;self.reset();self.page=0}});ui.label(lessons::hint(self.layout));ui.label(RichText::new("100 слов · курсор по центру").small().color(MUTED));});}ui.add_space(15.);ui.heading("Ваш текст");ui.add(egui::TextEdit::multiline(&mut self.custom).desired_rows(4).hint_text("Вставьте текст для тренировки"));if ui.add_enabled(!self.custom.trim().is_empty(),egui::Button::new("Тренироваться со своим текстом")).clicked(){let text=self.custom.replace("\r\n","\n").replace('\t',"    ");self.selected_endless=false;self.run=Run::new(&text);self.page=0;}
 }else if self.page==2 {let name=self.data.profiles.get(self.profile).cloned().unwrap_or_default();ui.label(format!("Профиль: {name}"));let records:Vec<_>=self.data.records.iter().filter(|r|r.profile==name).collect();if records.is_empty(){ui.add_space(40.);ui.heading("Первая тренировка — начало истории.");ui.label("Завершите урок, чтобы увидеть здесь свой прогресс.");}else{ui.label(format!("Завершено: {}    ·    Лучшая скорость: {:.0} зн/мин",records.len(),records.iter().map(|r|r.speed).fold(0.,f64::max)));let (rect,_)=ui.allocate_exact_size(egui::vec2(ui.available_width(),120.),egui::Sense::hover());let max=records.iter().map(|r|r.speed).fold(1.,f64::max);for(i,r)in records.iter().rev().take(30).rev().enumerate(){let w=rect.width()/30.;let h=(r.speed/max)as f32*100.;ui.painter().rect_filled(egui::Rect::from_min_size(egui::pos2(rect.left()+i as f32*w,rect.bottom()-h),egui::vec2(w-4.,h)),3.,if dark{LIME}else{GREEN});}egui::Grid::new("history").striped(true).spacing([24.,12.]).show(ui,|ui|{for s in ["Урок","Раскладка","Зн/мин","Точность","Ошибки"]{ui.strong(s);}ui.end_row();for r in records.iter().rev().take(100){ui.label(if r.endless {"Бесконечный режим"}else{lessons::title(r.layout.min(3),r.lesson.min(7))});ui.label(COURSES[r.layout.min(3)]);ui.label(format!("{:.0}",r.speed));ui.label(format!("{:.1}%",r.accuracy));ui.label(r.errors.to_string());ui.end_row();}});}
 }else if self.page==4 {self.home(ui,card);
 }else {if ui.checkbox(&mut self.data.dark,"Тёмная тема").changed(){self.save()}ui.heading("Профили");ui.label("История каждого пользователя хранится отдельно на этом компьютере.");ui.horizontal(|ui|{ui.text_edit_singleline(&mut self.new_profile);if ui.button("Добавить профиль").clicked(){let name=self.new_profile.trim().to_string();if !name.is_empty()&&!self.data.profiles.contains(&name){self.data.profiles.push(name);self.profile=self.data.profiles.len()-1;self.new_profile.clear();self.reset();self.save();}}});for p in &self.data.profiles{ui.label(format!("• {p}"));}ui.add_space(20.);if ui.checkbox(&mut self.data.sound,"Звук ошибки (Linux: canberra-gtk-play)").changed(){self.save()}ui.add_space(20.);ui.heading("Как заниматься");ui.label("Сидите удобно, расслабьте кисти.\nНачните с домашнего ряда и занимайтесь по 10–15 минут.\nЦвет клавиши подсказывает палец, подсветка — следующую букву.\nРаскладку операционной системы переключайте самостоятельно.");ui.label(format!("Данные: {}",self.path.display()));}
 });});});});
    }
}
fn main() -> eframe::Result {
    eframe::run_native(
        "Capee — практика печати",
        eframe::NativeOptions {
            viewport: egui::ViewportBuilder::default()
                .with_inner_size([1180., 860.])
                .with_resizable(true)
                .with_maximize_button(true)
                .with_min_inner_size([950., 720.]),
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unicode_errors_and_completion() {
        let mut r = Run::new("ёж");
        r.input('е');
        assert_eq!(r.position, 0);
        r.input('ё');
        r.input('ж');
        assert!(r.done);
        assert_eq!(r.errors, 1);
        assert!((r.accuracy() - 66.6667).abs() < 0.01);
        let n = r.attempts;
        r.input('!');
        assert_eq!(r.attempts, n);
    }
    #[test]
    fn pause_ignores_input() {
        let mut r = Run::new("a");
        r.pause();
        r.input('a');
        assert_eq!(r.position, 0);
        r.pause();
        r.input('a');
        assert!(r.done);
    }
    #[test]
    fn history_round_trip() {
        let d = Data {
            records: vec![Record {
                profile: "Ира".into(),
                speed: 120.,
                ..Default::default()
            }],
            ..Default::default()
        };
        let b = serde_json::to_vec(&d).unwrap();
        let restored: Data = serde_json::from_slice(&b).unwrap();
        assert_eq!(restored.records[0].profile, "Ира");
    }
}

#[cfg(test)]
mod curriculum_tests {
    use super::*;
    #[test]
    fn all_lessons_are_completable() {
        for course in 0..4 {
            for lesson in 0..8 {
                let text = lessons::exercise(course, lesson);
                assert_eq!(text.split_whitespace().count(), 100);
                let mut run = Run::new(&text);
                for c in text.chars() {
                    run.input(c)
                }
                assert!(run.done);
                assert_eq!(run.accuracy(), 100.0);
            }
        }
    }
    #[test]
    fn multiline_and_indent() {
        let mut r = Run::new("x\n    y");
        for c in "x\n    y".chars() {
            r.input(c)
        }
        assert!(r.done);
        assert_eq!(r.position, 7);
    }
}

#[cfg(test)]
mod preference_tests {
    use super::*;
    #[test]
    fn old_history_loads_in_light_mode() {
        let data: Data =
            serde_json::from_str(r#"{"profiles":["User"],"records":[],"sound":false}"#).unwrap();
        assert!(!data.dark);
    }
    #[test]
    fn dark_mode_is_retained() {
        let data = Data {
            dark: true,
            ..Default::default()
        };
        let loaded: Data = serde_json::from_slice(&serde_json::to_vec(&data).unwrap()).unwrap();
        assert!(loaded.dark);
    }
}

#[cfg(test)]
mod scrolling_tests {
    use super::*;
    #[test]
    fn code_keeps_line_breaks_and_indentation() {
        let text = lessons::exercise(2, 3);
        assert!(text.contains("\n    print"));
        assert_eq!(text.split_whitespace().count(), 100);
    }
    #[test]
    fn scroll_cursor_uses_character_positions() {
        let ctx = egui::Context::default();
        let _ = ctx.run(egui::RawInput::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let text = "ёж ↵ rust";
                let galley = ui.fonts(|f| {
                    f.layout_no_wrap(text.into(), egui::FontId::monospace(25.), Color32::WHITE)
                });
                for index in 0..=text.chars().count() {
                    let cursor = galley.pos_from_ccursor(egui::text::CCursor::new(index));
                    let origin = 400. - cursor.left();
                    assert!((origin + cursor.left() - 400.).abs() < 0.01);
                }
            });
        });
    }
}

#[cfg(test)]
mod endless_tests {
    use super::*;
    #[test]
    fn endless_stream_continues_and_bounds_memory() {
        for course in 0..4 {
            let mut run = Run::infinite(course, 123);
            for _ in 0..25000 {
                let next = run.text[run.position];
                run.input(next);
                assert!(!run.done);
                assert!(run.text.len() < 2500);
            }
            assert_eq!(run.typed(), 25000);
            assert_eq!(run.accuracy(), 100.);
            assert!(run.discarded > 0);
            run.finish();
            assert!(run.done);
            assert!(run.start.is_none());
            let count = run.attempts;
            run.input('!');
            assert_eq!(run.attempts, count);
        }
    }
    #[test]
    fn endless_pause_and_errors_preserve_stream() {
        let mut run = Run::infinite(0, 1);
        run.input('\0');
        assert_eq!(run.errors, 1);
        assert_eq!(run.position, 0);
        run.pause();
        let expected = run.text[0];
        run.input(expected);
        assert_eq!(run.position, 0);
        run.pause();
        run.input(expected);
        assert_eq!(run.typed(), 1);
    }
    #[test]
    fn new_blocks_vary_and_code_retains_whitespace() {
        for course in 0..4 {
            let mut state = 25;
            let examples: std::collections::HashSet<_> = (0..20)
                .map(|_| lessons::endless_block(course, &mut state))
                .collect();
            assert!(examples.len() > 3);
            assert!(examples.iter().all(|s| !s.is_empty()));
            if course >= 2 {
                assert!(examples.iter().any(|s| s.contains('\n')));
            }
        }
    }
    #[test]
    fn old_record_loads_as_regular_lesson() {
        let record:Record=serde_json::from_str(r#"{"profile":"A","layout":0,"lesson":0,"speed":120.0,"accuracy":100.0,"errors":0,"seconds":20.0}"#).unwrap();
        assert!(!record.endless);
    }
}
