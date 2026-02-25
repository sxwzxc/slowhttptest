// SlowHTTPTest GUI - A graphical frontend for slowhttptest
// Allows users to configure and launch slowhttptest from a visual interface.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fmt::Write as FmtWrite;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use eframe::egui;

fn main() -> eframe::Result {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("SlowHTTPTest GUI")
            .with_min_inner_size([800.0, 600.0])
            .with_inner_size([1000.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "SlowHTTPTest GUI",
        options,
        Box::new(|_cc| Ok(Box::new(App::default()))),
    )
}

// ──────────────────────────────────────────────────────────────────────────────
// Data model
// ──────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum TestMode {
    SlowHeaders,
    SlowBody,
    RangeAttack,
    SlowRead,
}

impl TestMode {
    fn flag(&self) -> &'static str {
        match self {
            TestMode::SlowHeaders => "-H",
            TestMode::SlowBody => "-B",
            TestMode::RangeAttack => "-R",
            TestMode::SlowRead => "-X",
        }
    }
    fn label(&self) -> &'static str {
        match self {
            TestMode::SlowHeaders => "Slow Headers (Slowloris)  -H",
            TestMode::SlowBody => "Slow Body (R-U-Dead-Yet)  -B",
            TestMode::RangeAttack => "Range Attack (Apache Killer)  -R",
            TestMode::SlowRead => "Slow Read  -X",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
enum ProxyMode {
    None,
    Http,
    Probe,
}

struct App {
    // General
    url: String,
    test_mode: TestMode,
    connections: String,
    rate: String,
    duration: String,
    interval: String,
    verb: String,
    content_length: String,
    max_random_data_len: String,

    // HTTP options
    content_type: String,
    accept: String,
    cookie: String,
    custom_header: String,

    // Proxy
    proxy_mode: ProxyMode,
    proxy_addr: String,

    // Reporting
    generate_stats: bool,
    stats_file_prefix: String,
    verbosity: String,

    // Range attack specific
    range_start: String,
    range_limit: String,

    // Slow read specific
    pipeline_factor: String,
    probe_interval: String,
    read_interval: String,
    read_len: String,
    window_lower: String,
    window_upper: String,

    // UI state
    output: Arc<Mutex<String>>,
    running: Arc<Mutex<bool>>,
    custom_binary_path: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            url: "http://localhost/".to_owned(),
            test_mode: TestMode::SlowHeaders,
            connections: "50".to_owned(),
            rate: "50".to_owned(),
            duration: "240".to_owned(),
            interval: "10".to_owned(),
            verb: String::new(),
            content_length: "4096".to_owned(),
            max_random_data_len: "32".to_owned(),

            content_type: String::new(),
            accept: String::new(),
            cookie: String::new(),
            custom_header: String::new(),

            proxy_mode: ProxyMode::None,
            proxy_addr: String::new(),

            generate_stats: false,
            stats_file_prefix: "stats".to_owned(),
            verbosity: "1".to_owned(),

            range_start: "5".to_owned(),
            range_limit: "2000".to_owned(),

            pipeline_factor: "1".to_owned(),
            probe_interval: "5".to_owned(),
            read_interval: "1".to_owned(),
            read_len: "5".to_owned(),
            window_lower: "1".to_owned(),
            window_upper: "512".to_owned(),

            output: Arc::new(Mutex::new(String::new())),
            running: Arc::new(Mutex::new(false)),
            custom_binary_path: String::new(),
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// Command builder
// ──────────────────────────────────────────────────────────────────────────────

impl App {
    fn build_args(&self) -> Vec<String> {
        let mut args: Vec<String> = Vec::new();

        args.push(self.test_mode.flag().to_owned());
        args.push("-u".to_owned());
        args.push(self.url.clone());

        push_opt_num(&mut args, "-c", &self.connections, 50);
        push_opt_num(&mut args, "-r", &self.rate, 50);
        push_opt_num(&mut args, "-l", &self.duration, 240);
        push_opt_num(&mut args, "-i", &self.interval, 10);
        push_opt_num(&mut args, "-s", &self.content_length, 4096);
        push_opt_num(&mut args, "-x", &self.max_random_data_len, 32);

        if !self.verb.trim().is_empty() {
            args.push("-t".to_owned());
            args.push(self.verb.trim().to_owned());
        }

        if !self.content_type.trim().is_empty() {
            args.push("-f".to_owned());
            args.push(self.content_type.trim().to_owned());
        }
        if !self.accept.trim().is_empty() {
            args.push("-m".to_owned());
            args.push(self.accept.trim().to_owned());
        }
        if !self.cookie.trim().is_empty() {
            args.push("-j".to_owned());
            args.push(self.cookie.trim().to_owned());
        }
        if !self.custom_header.trim().is_empty() {
            args.push("-1".to_owned());
            args.push(self.custom_header.trim().to_owned());
        }

        match self.proxy_mode {
            ProxyMode::None => {}
            ProxyMode::Http => {
                if !self.proxy_addr.trim().is_empty() {
                    args.push("-d".to_owned());
                    args.push(self.proxy_addr.trim().to_owned());
                }
            }
            ProxyMode::Probe => {
                if !self.proxy_addr.trim().is_empty() {
                    args.push("-e".to_owned());
                    args.push(self.proxy_addr.trim().to_owned());
                }
            }
        }

        push_opt_num(&mut args, "-p", &self.probe_interval, 5);

        if self.generate_stats {
            args.push("-g".to_owned());
            if !self.stats_file_prefix.trim().is_empty() {
                args.push("-o".to_owned());
                args.push(self.stats_file_prefix.trim().to_owned());
            }
        }

        push_opt_num(&mut args, "-v", &self.verbosity, 1);

        match self.test_mode {
            TestMode::RangeAttack => {
                push_opt_num(&mut args, "-a", &self.range_start, 5);
                push_opt_num(&mut args, "-b", &self.range_limit, 2000);
            }
            TestMode::SlowRead => {
                push_opt_num(&mut args, "-k", &self.pipeline_factor, 1);
                push_opt_num(&mut args, "-n", &self.read_interval, 1);
                push_opt_num(&mut args, "-w", &self.window_lower, 1);
                push_opt_num(&mut args, "-y", &self.window_upper, 512);
                push_opt_num(&mut args, "-z", &self.read_len, 5);
            }
            _ => {}
        }

        args
    }

    fn build_command_preview(&self) -> String {
        let binary = self.effective_binary();
        let args = self.build_args();
        let mut s = binary;
        for a in &args {
            if a.contains(' ') {
                write!(s, " \"{}\"", a).ok();
            } else {
                write!(s, " {}", a).ok();
            }
        }
        s
    }

    fn effective_binary(&self) -> String {
        if self.custom_binary_path.trim().is_empty() {
            "slowhttptest".to_owned()
        } else {
            self.custom_binary_path.trim().to_owned()
        }
    }

    fn launch(&mut self) {
        if *self.running.lock().unwrap() {
            return;
        }

        let binary = self.effective_binary();
        let args = self.build_args();
        let output = Arc::clone(&self.output);
        let running = Arc::clone(&self.running);

        {
            let mut o = output.lock().unwrap();
            o.clear();
            writeln!(o, "$ {} {}", binary, args.join(" ")).ok();
        }
        *running.lock().unwrap() = true;

        thread::spawn(move || {
            let result = Command::new(&binary)
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn();

            match result {
                Err(e) => {
                    let mut o = output.lock().unwrap();
                    writeln!(o, "[ERROR] Failed to start '{}': {}", binary, e).ok();
                    writeln!(o, "  Make sure slowhttptest is installed and in your PATH,").ok();
                    writeln!(o, "  or set a custom binary path above.").ok();
                }
                Ok(mut child) => {
                    // Stream stderr (slowhttptest writes most output to stderr)
                    if let Some(stderr) = child.stderr.take() {
                        let output2 = Arc::clone(&output);
                        thread::spawn(move || {
                            for line in BufReader::new(stderr).lines() {
                                if let Ok(l) = line {
                                    let mut o = output2.lock().unwrap();
                                    writeln!(o, "{}", l).ok();
                                }
                            }
                        });
                    }
                    // Stream stdout
                    if let Some(stdout) = child.stdout.take() {
                        for line in BufReader::new(stdout).lines() {
                            if let Ok(l) = line {
                                let mut o = output.lock().unwrap();
                                writeln!(o, "{}", l).ok();
                            }
                        }
                    }
                    let status = child.wait();
                    let mut o = output.lock().unwrap();
                    match status {
                        Ok(s) => writeln!(o, "\n[Process exited with status: {}]", s).ok(),
                        Err(e) => writeln!(o, "\n[Wait error: {}]", e).ok(),
                    };
                }
            }
            *running.lock().unwrap() = false;
        });
    }
}

fn push_opt_num(args: &mut Vec<String>, flag: &str, val: &str, default: i64) {
    if let Ok(n) = val.trim().parse::<i64>() {
        if n != default {
            args.push(flag.to_owned());
            args.push(n.to_string());
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────────
// UI
// ──────────────────────────────────────────────────────────────────────────────

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Request repaint while test is running so output scrolls in real time
        if *self.running.lock().unwrap() {
            ctx.request_repaint();
        }

        egui::TopBottomPanel::top("header").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("⚡ SlowHTTPTest GUI");
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.hyperlink_to(
                        "GitHub",
                        "https://github.com/shekyan/slowhttptest",
                    );
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // Split into left config panel and right output panel
            ui.columns(2, |cols| {
                // ── LEFT: configuration ──────────────────────────────────────
                egui::ScrollArea::vertical()
                    .id_salt("config_scroll")
                    .show(&mut cols[0], |ui| {
                        self.draw_config(ui);
                    });

                // ── RIGHT: command preview + output ──────────────────────────
                cols[1].vertical(|ui| {
                    self.draw_output(ui);
                });
            });
        });
    }
}

impl App {
    fn draw_config(&mut self, ui: &mut egui::Ui) {
        // ── Test mode ────────────────────────────────────────────────────────
        egui::CollapsingHeader::new("🔧 Test Mode")
            .default_open(true)
            .show(ui, |ui| {
                for mode in [
                    TestMode::SlowHeaders,
                    TestMode::SlowBody,
                    TestMode::RangeAttack,
                    TestMode::SlowRead,
                ] {
                    ui.radio_value(&mut self.test_mode, mode.clone(), mode.label());
                }
            });

        ui.separator();

        // ── Target URL ───────────────────────────────────────────────────────
        egui::CollapsingHeader::new("🌐 Target")
            .default_open(true)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label("URL:");
                    ui.add(egui::TextEdit::singleline(&mut self.url).hint_text("http://target/"));
                });
                ui.horizontal(|ui| {
                    ui.label("Binary path:");
                    ui.add(
                        egui::TextEdit::singleline(&mut self.custom_binary_path)
                            .hint_text("slowhttptest  (leave blank to use PATH)"),
                    );
                });
            });

        ui.separator();

        // ── Connection settings ──────────────────────────────────────────────
        egui::CollapsingHeader::new("🔗 Connection Settings")
            .default_open(true)
            .show(ui, |ui| {
                egui::Grid::new("conn_grid")
                    .num_columns(4)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Connections (-c):");
                        ui.add(num_edit(&mut self.connections));
                        ui.label("Rate/s (-r):");
                        ui.add(num_edit(&mut self.rate));
                        ui.end_row();

                        ui.label("Duration s (-l):");
                        ui.add(num_edit(&mut self.duration));
                        ui.label("Interval s (-i):");
                        ui.add(num_edit(&mut self.interval));
                        ui.end_row();

                        ui.label("Content-Length (-s):");
                        ui.add(num_edit(&mut self.content_length));
                        ui.label("Max rand data (-x):");
                        ui.add(num_edit(&mut self.max_random_data_len));
                        ui.end_row();
                    });
            });

        ui.separator();

        // ── Mode-specific options ────────────────────────────────────────────
        match self.test_mode {
            TestMode::RangeAttack => {
                egui::CollapsingHeader::new("📐 Range Attack Options")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::Grid::new("range_grid")
                            .num_columns(4)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Range start (-a):");
                                ui.add(num_edit(&mut self.range_start));
                                ui.label("Range limit (-b):");
                                ui.add(num_edit(&mut self.range_limit));
                                ui.end_row();
                            });
                    });
                ui.separator();
            }
            TestMode::SlowRead => {
                egui::CollapsingHeader::new("📖 Slow Read Options")
                    .default_open(true)
                    .show(ui, |ui| {
                        egui::Grid::new("slowread_grid")
                            .num_columns(4)
                            .spacing([8.0, 4.0])
                            .show(ui, |ui| {
                                ui.label("Pipeline factor (-k):");
                                ui.add(num_edit(&mut self.pipeline_factor));
                                ui.label("Read interval s (-n):");
                                ui.add(num_edit(&mut self.read_interval));
                                ui.end_row();

                                ui.label("Window min (-w):");
                                ui.add(num_edit(&mut self.window_lower));
                                ui.label("Window max (-y):");
                                ui.add(num_edit(&mut self.window_upper));
                                ui.end_row();

                                ui.label("Read bytes (-z):");
                                ui.add(num_edit(&mut self.read_len));
                                ui.end_row();
                            });
                    });
                ui.separator();
            }
            _ => {}
        }

        // ── HTTP options ─────────────────────────────────────────────────────
        egui::CollapsingHeader::new("📋 HTTP Options")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("http_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("HTTP Verb (-t):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.verb)
                                .hint_text("GET / POST  (auto)"),
                        );
                        ui.end_row();

                        ui.label("Content-Type (-f):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.content_type)
                                .hint_text("application/x-www-form-urlencoded"),
                        );
                        ui.end_row();

                        ui.label("Accept (-m):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.accept)
                                .hint_text("text/html;q=0.9,…"),
                        );
                        ui.end_row();

                        ui.label("Cookie (-j):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.cookie)
                                .hint_text("name=value; name2=value2"),
                        );
                        ui.end_row();

                        ui.label("Custom Header (-1):");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.custom_header)
                                .hint_text("X-Custom: value"),
                        );
                        ui.end_row();
                    });
            });

        ui.separator();

        // ── Proxy ────────────────────────────────────────────────────────────
        egui::CollapsingHeader::new("🔀 Proxy")
            .default_open(false)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.radio_value(&mut self.proxy_mode, ProxyMode::None, "None");
                    ui.radio_value(&mut self.proxy_mode, ProxyMode::Http, "HTTP proxy (-d)");
                    ui.radio_value(
                        &mut self.proxy_mode,
                        ProxyMode::Probe,
                        "Probe proxy (-e)",
                    );
                });
                if self.proxy_mode != ProxyMode::None {
                    ui.horizontal(|ui| {
                        ui.label("host:port:");
                        ui.add(
                            egui::TextEdit::singleline(&mut self.proxy_addr)
                                .hint_text("127.0.0.1:8080"),
                        );
                    });
                }
                egui::Grid::new("proxy_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Probe timeout s (-p):");
                        ui.add(num_edit(&mut self.probe_interval));
                        ui.end_row();
                    });
            });

        ui.separator();

        // ── Reporting ────────────────────────────────────────────────────────
        egui::CollapsingHeader::new("📊 Reporting")
            .default_open(false)
            .show(ui, |ui| {
                egui::Grid::new("report_grid")
                    .num_columns(2)
                    .spacing([8.0, 4.0])
                    .show(ui, |ui| {
                        ui.label("Generate stats (-g):");
                        ui.checkbox(&mut self.generate_stats, "");
                        ui.end_row();

                        if self.generate_stats {
                            ui.label("Output prefix (-o):");
                            ui.add(
                                egui::TextEdit::singleline(&mut self.stats_file_prefix)
                                    .hint_text("stats"),
                            );
                            ui.end_row();
                        }

                        ui.label("Verbosity (-v) 0-4:");
                        ui.add(num_edit(&mut self.verbosity));
                        ui.end_row();
                    });
            });

        ui.separator();

        // ── Launch button ────────────────────────────────────────────────────
        let is_running = *self.running.lock().unwrap();
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            let btn = egui::Button::new(if is_running {
                "⏳ Running…"
            } else {
                "▶  Run Test"
            })
            .fill(if is_running {
                egui::Color32::from_rgb(100, 100, 100)
            } else {
                egui::Color32::from_rgb(0, 160, 80)
            });

            if ui.add_enabled(!is_running, btn).clicked() {
                self.launch();
            }

            if is_running {
                ui.spinner();
                ui.label("Test in progress…");
            }
        });
        ui.add_space(8.0);
    }

    fn draw_output(&mut self, ui: &mut egui::Ui) {
        ui.label(egui::RichText::new("Command Preview").strong());
        let preview = self.build_command_preview();
        egui::ScrollArea::horizontal()
            .id_salt("preview_scroll")
            .max_height(56.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut preview.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(2),
                );
            });

        ui.add_space(4.0);
        ui.separator();

        ui.horizontal(|ui| {
            ui.label(egui::RichText::new("Output").strong());
            if ui.small_button("Clear").clicked() {
                self.output.lock().unwrap().clear();
            }
        });

        let output_text = self.output.lock().unwrap().clone();
        let available = ui.available_height();
        egui::ScrollArea::vertical()
            .id_salt("output_scroll")
            .max_height(available)
            .stick_to_bottom(true)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(&mut output_text.as_str())
                        .font(egui::TextStyle::Monospace)
                        .desired_width(f32::INFINITY)
                        .desired_rows(20),
                );
            });
    }
}

fn num_edit(s: &mut String) -> egui::TextEdit<'_> {
    egui::TextEdit::singleline(s).desired_width(72.0)
}
