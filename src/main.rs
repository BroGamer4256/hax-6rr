#![feature(let_chains)]
use anyhow::Result;
use eframe::egui;
use num_traits::FromPrimitive;
use url::Url;

pub mod car;
pub mod items;
pub mod useritems;
pub mod wm;

fn main() {
	let native_options = eframe::NativeOptions {
		viewport: egui::ViewportBuilder::default().with_drag_and_drop(true),
		..Default::default()
	};

	let runtime = tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap();
	eframe::run_native(
		"6RR hax",
		native_options,
		Box::new(|cc| {
			let mut fonts = egui::FontDefinitions::default();
			fonts.font_data.insert(
				String::from("NotoSansJP"),
				egui::FontData::from_static(include_bytes!(
					"../fonts/NotoSansJP-VariableFont_wght.ttf"
				)),
			);

			fonts
				.families
				.get_mut(&egui::FontFamily::Proportional)
				.unwrap()
				.insert(0, String::from("NotoSansJP"));

			fonts
				.families
				.get_mut(&egui::FontFamily::Monospace)
				.unwrap()
				.insert(0, String::from("NotoSansJP"));

			cc.egui_ctx.set_fonts(fonts);

			Box::new(App {
				runtime,
				server_buf: String::new(),
				server: None,
				cars: Vec::new(),
				car: None,
				car_setting: None,
				car_items: Vec::new(),
				car_play_count: 0,
				car_odometer: 0,
				vs_cool_or_wild: 0,
				vs_smooth_or_rough: 0,
				vs_play_count: 0,
				sub_menu: None,
				user_items: Vec::new(),
			})
		}),
	)
	.unwrap();
}

struct App {
	runtime: tokio::runtime::Runtime,
	server_buf: String,
	server: Option<Url>,
	cars: Vec<wm::Car>,
	car: Option<wm::Car>,
	car_setting: Option<wm::CarSetting>,
	car_items: Vec<wm::CarItem>,
	car_play_count: u32,
	car_odometer: u32,
	vs_cool_or_wild: i32,
	vs_smooth_or_rough: i32,
	vs_play_count: u32,
	sub_menu: Option<SubMenu>,
	user_items: Vec<wm::UserItem>,
}

enum SubMenu {
	Items(items::ItemMenu),
	Car(car::CarMenu),
	UserItems(useritems::UserItems),
}

impl eframe::App for App {
	fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
		if !self.cars.is_empty() {
			egui::TopBottomPanel::top("TopPanel").show(ctx, |ui| {
				egui::Grid::new("TopGrid").num_columns(2).show(ui, |ui| {
					if ui.button("Back").clicked() {
						if let Some(sub_menu) = &mut self.sub_menu {
							let want_to_exit = match sub_menu {
								SubMenu::Items(items) => items.back(),
								SubMenu::Car(car) => car.back(),
								SubMenu::UserItems(useritems) => useritems.back(),
							};
							if want_to_exit {
								self.sub_menu = None;
							}
						} else if self.car.is_none() {
							self.cars = Vec::new();
						} else {
							self.car = None;
						}
					}
					if let Some(car) = &self.car
						&& let Some(car_name) = wm::Cars::from_u32(car.visual_model())
					{
						ui.label(format!("{} ({})", car.name(), car_name));
					}
					ui.end_row();
				});
			});
		}

		egui::CentralPanel::default().show(ctx, |ui| {
			egui::ScrollArea::vertical().show(ui, |ui| {
				if let Some(sub_menu) = &mut self.sub_menu {
					match sub_menu {
						SubMenu::Items(menu) => menu.update(
							ui,
							&self.runtime,
							self.server.as_ref().unwrap(),
							self.car.as_ref().unwrap(),
							&mut self.car_items,
						),
						SubMenu::Car(menu) => menu.update(
							ui,
							&self.runtime,
							self.server.as_ref().unwrap(),
							self.car.as_mut().unwrap(),
							self.car_setting.as_mut().unwrap(),
							&self.car_items,
						),
						SubMenu::UserItems(menu) => menu.update(
							ui,
							&self.runtime,
							self.server.as_ref().unwrap(),
							&mut self.user_items,
							self.cars.first().as_ref().unwrap(),
							self.car_odometer,
						),
					}
				} else {
					self.runtime.block_on(async {
						if self.server.is_none() {
							if ui
								.add(
									egui::TextEdit::singleline(&mut self.server_buf)
										.hint_text("Server URL"),
								)
								.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter))
							{
								if !self.server_buf.starts_with("http://")
									|| !self.server_buf.starts_with("https://")
								{
									self.server_buf.insert_str(0, "https://")
								}
								if let Some(mut server) = Url::parse(&self.server_buf).ok() {
									if server.port().is_none()
										&& server.set_port(Some(9002)).is_err()
									{
										return;
									}
									self.server = Some(server);
								}
							}
						} else if self.cars.is_empty() {
							let user = wait_user(ui, ctx, self.server.as_ref().unwrap()).await;
							if let Some(user) = user {
								self.cars = user.cars;
								self.user_items = user.unused_car_tickets;
							}
						} else if self.car.is_none() {
							let car =
								wait_select_car(&self.cars, ui, self.server.as_ref().unwrap())
									.await;
							if let Some(car) = car {
								self.car = Some(car.car);
								self.car_setting = Some(car.setting);
								self.car_items = car.owned_items;
								self.car_play_count = car.play_count;
								self.car_odometer = car.odometer;
								self.vs_cool_or_wild = car.vs_cool_or_wild;
								self.vs_smooth_or_rough = car.vs_smooth_or_rough;
								self.vs_play_count = car.vs_play_count;
							}
							if ui.button("User Items").clicked() {
								self.sub_menu = Some(SubMenu::UserItems(useritems::UserItems {
									selected_category: None,
									new_item_buf: None,
								}));
							}
						} else if self.sub_menu.is_none() {
							if ui.button("Items").clicked() {
								self.sub_menu = Some(SubMenu::Items(items::ItemMenu {
									new_item_buf: String::new(),
									selected_category: None,
								}));
							} else if ui.button("Car").clicked() {
								self.sub_menu = Some(SubMenu::Car(car::CarMenu {
									play_count: self.car_play_count,
									odometer: self.car_odometer,
									vs_cool_or_wild: self.vs_cool_or_wild,
									vs_smooth_or_rough: self.vs_smooth_or_rough,
									vs_play_count: self.vs_play_count,
									odometer_buf: self.car_odometer.to_string(),
									cool_or_wild_buf: self.vs_cool_or_wild.to_string(),
									smooth_or_rough_buf: self.vs_smooth_or_rough.to_string(),
								}));
							}
						}
					});
				}
			});
		});
	}
}

async fn wait_user(
	ui: &mut egui::Ui,
	ctx: &egui::Context,
	server: &Url,
) -> Option<wm::LoadUserResponse> {
	ui.heading("Drop card.ini onto window");

	for file in ctx.input(|i| i.raw.dropped_files.clone()) {
		let path = match file.path {
			Some(path) => path,
			None => continue,
		};
		let path = match path.to_str() {
			Some(path) => path,
			None => continue,
		};
		let user = load_user(path, server).await;
		if let Ok(user) = user {
			return Some(user);
		}
	}

	None
}

async fn wait_select_car(
	cars: &[wm::Car],
	ui: &mut egui::Ui,
	server: &Url,
) -> Option<wm::LoadCarResponse> {
	for car in cars.iter() {
		if ui
			.button(format!(
				"{} ({})",
				car.name(),
				wm::Cars::from_u32(car.visual_model())?
			))
			.clicked()
		{
			let car = load_car(car.car_id(), server).await;
			if let Ok(car) = car {
				return Some(car);
			}
		}
	}

	None
}

async fn load_user(ini_path: &str, server: &Url) -> Result<wm::LoadUserResponse> {
	#[derive(serde::Deserialize)]
	struct Card {
		#[serde(rename = "accessCode")]
		access_code: String,
		#[serde(rename = "chipId")]
		chip_id: String,
	}

	#[derive(serde::Deserialize)]
	struct CardHolder {
		card: Card,
	}

	let ini = tokio::fs::read_to_string(ini_path).await?;

	let card: CardHolder = serde_ini::from_str(&ini)?;

	let req = wm::LoadUserRequest {
		card_chip_id: Some(card.card.chip_id),
		access_code: Some(card.card.access_code),
		max_cars: 255,
		create_user: Some(false),
		..Default::default()
	};

	wm::send_request(req, server, "method/load_user").await
}

async fn load_car(car_id: u32, server: &Url) -> Result<wm::LoadCarResponse> {
	let req = wm::LoadCarRequest {
		car_id,
		..Default::default()
	};

	wm::send_request(req, server, "method/load_car").await
}
