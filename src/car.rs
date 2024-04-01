use crate::*;
use anyhow::Result;
use eframe::egui;
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::{FromPrimitive, ToPrimitive};
use strum::{EnumIter, IntoEnumIterator};
use url::Url;

pub struct CarMenu {
	pub play_count: u32,
	pub odometer: u32,
	pub vs_cool_or_wild: i32,
	pub vs_smooth_or_rough: i32,
	pub vs_play_count: u32,
	pub odometer_buf: String,
	pub cool_or_wild_buf: String,
	pub smooth_or_rough_buf: String,
}

impl CarMenu {
	pub fn update(
		&mut self,
		ui: &mut egui::Ui,
		runtime: &tokio::runtime::Runtime,
		server: &Url,
		car: &mut wm::Car,
		car_settings: &mut wm::CarSetting,
		car_items: &[wm::CarItem],
	) {
		runtime.block_on(async {
			if ui.button("Save").clicked() {
				_ = save_car(
					server,
					car,
					car_settings,
					self.play_count,
					self.odometer_buf.parse().unwrap_or(self.odometer),
					self.cool_or_wild_buf
						.parse()
						.unwrap_or(self.vs_cool_or_wild),
					self.smooth_or_rough_buf
						.parse()
						.unwrap_or(self.vs_smooth_or_rough),
					self.vs_play_count,
				)
				.await;
			}

			egui::Grid::new("CarGrid").num_columns(2).show(ui, |ui| {
				set_car_class(ui, car);
				set_volume(ui, car_settings);
				set_bgm(ui, car_settings, car_items);
				set_meter(ui, car_settings, car_items);
				set_nameplate(ui, car_settings, car_items);

				ui.label("Navigation map");
				ui.add(egui::Checkbox::without_text(
					&mut car_settings.navigation_map,
				));
				ui.end_row();

				ui.label("Retire");
				ui.add(egui::Checkbox::without_text(&mut car_settings.retire));
				ui.end_row();

				ui.label("Manual transmission");
				ui.add(egui::Checkbox::without_text(&mut car_settings.transmission));
				ui.end_row();

				ui.label("Third person");
				ui.add(egui::Checkbox::without_text(&mut car_settings.view));
				ui.end_row();

				ui.label("Title");
				ui.add(egui::TextEdit::singleline(&mut car.title));
				ui.end_row();

				ui.label("Odometer");
				ui.add(egui::TextEdit::singleline(&mut self.odometer_buf));
				ui.end_row();

				ui.label("Cool Or Wild");
				ui.add(egui::TextEdit::singleline(&mut self.cool_or_wild_buf));
				ui.end_row();

				ui.label("Smooth Or Rough");
				ui.add(egui::TextEdit::singleline(&mut self.smooth_or_rough_buf));
				ui.end_row();
			});
		});
	}

	pub fn back(&mut self) -> bool {
		true
	}
}

fn set_volume(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting) {
	#[derive(FromPrimitive, ToPrimitive, EnumIter)]
	enum Volume {
		Muted = 0,
		Minimum = 1,
		Medium = 2,
		Maximum = 3,
	}

	impl ToString for Volume {
		fn to_string(&self) -> String {
			match self {
				Volume::Muted => String::from("Muted"),
				Volume::Minimum => String::from("Minimum"),
				Volume::Medium => String::from("Medium"),
				Volume::Maximum => String::from("Maximum"),
			}
		}
	}

	ui.label("Volume");
	egui::ComboBox::from_id_source("VolumeComboBox")
		.selected_text(
			Volume::from_u32(car_settings.volume)
				.unwrap_or(Volume::Medium)
				.to_string(),
		)
		.show_ui(ui, |ui| {
			for volume in Volume::iter() {
				ui.selectable_value(
					&mut car_settings.volume,
					volume.to_u32().unwrap_or(2),
					volume.to_string(),
				);
			}
		});
	ui.end_row();
}

fn set_bgm(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting, car_items: &[wm::CarItem]) {
	ui.label("Bgm");

	let selected = match wm::Bgms::from_u32(car_settings.bgm) {
		Some(bgm) => bgm.to_string(),
		None => String::from("WMMT 6/6R/6RR"),
	};
	egui::ComboBox::from_id_source("BgmComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car_settings.bgm, 0, "WMMT 6/6R/6RR");
			for bgm in wm::Bgms::iter() {
				if car_items
					.iter()
					.filter(|item| {
						item.category == wm::ItemCategory::CatBgm.into()
							&& Some(item.item_id) == bgm.to_u32()
					})
					.collect::<Vec<_>>()
					.len() == 1
				{
					ui.selectable_value(
						&mut car_settings.bgm,
						bgm.to_u32().unwrap_or(0),
						bgm.to_string(),
					);
				}
			}
		});
	ui.end_row();
}

fn set_meter(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting, car_items: &[wm::CarItem]) {
	ui.label("Meter");

	let selected = match wm::Meters::from_u32(car_settings.meter) {
		Some(meter) => meter.to_string(),
		None => String::from("Stock"),
	};
	egui::ComboBox::from_id_source("MeterComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car_settings.meter, 0, "Stock");
			for meter in wm::Meters::iter() {
				if car_items
					.iter()
					.filter(|item| {
						item.category == wm::ItemCategory::CatMeter.into()
							&& Some(item.item_id) == meter.to_u32()
					})
					.collect::<Vec<_>>()
					.len() == 1
				{
					ui.selectable_value(
						&mut car_settings.meter,
						meter.to_u32().unwrap_or(0),
						meter.to_string(),
					);
				}
			}
		});
	ui.end_row();
}

fn set_nameplate(ui: &mut egui::Ui, car_settings: &mut wm::CarSetting, car_items: &[wm::CarItem]) {
	ui.label("Nameplate");

	let selected = match wm::Nameplates::from_u32(car_settings.nameplate) {
		Some(nameplate) => nameplate.to_string(),
		None => String::from("Stock"),
	};
	egui::ComboBox::from_id_source("NameplateComboBox")
		.selected_text(selected)
		.show_ui(ui, |ui| {
			ui.selectable_value(&mut car_settings.nameplate, 0, "Stock");
			for nameplate in wm::Nameplates::iter() {
				if !car_items
					.iter()
					.filter(|item| {
						item.category == wm::ItemCategory::CatNamePlate.into()
							&& Some(item.item_id) == nameplate.to_u32()
					})
					.collect::<Vec<_>>()
					.is_empty()
				{
					ui.selectable_value(
						&mut car_settings.nameplate,
						nameplate.to_u32().unwrap_or(0),
						nameplate.to_string(),
					);
				}
			}
		});
	ui.end_row();
}

fn set_car_class(ui: &mut egui::Ui, car: &mut wm::Car) {
	const CLASSES: &[&str] = &["C", "B", "A", "S", "SS", "SSS", "SSSS", "SSSSS"];

	fn get_class(class: u32) -> String {
		if class == 1 {
			return String::from("N");
		} else if class >= 74 {
			return String::from("SSSSSS");
		}

		let class = class as usize - 2;
		let numbers: Vec<u32> = (1..=9).rev().collect();
		format!("{}{}", CLASSES[class / 9], numbers[class % 9])
	}

	ui.label("Class");
	egui::ComboBox::from_id_source("ClassComboBox")
		.selected_text(get_class(car.level))
		.show_ui(ui, |ui| {
			for class in 1..=(CLASSES.len() as u32 * 9 + 2) {
				ui.selectable_value(&mut car.level, class, get_class(class));
			}
		});
	ui.end_row();
}

async fn save_car(
	server: &Url,
	car: &wm::Car,
	car_settings: &wm::CarSetting,
	play_count: u32,
	odometer: u32,
	vs_cool_or_wild: i32,
	vs_smooth_or_rough: i32,
	vs_play_count: u32,
) -> Result<wm::SaveGameResultResponse> {
	let req = wm::SaveGameResultRequest {
		car_id: car.car_id(),
		game_mode: wm::GameMode::ModeVsBattle.into(),
		played_at: car.last_played_at(),
		play_count,
		car: Some(car.clone()),
		setting: Some(car_settings.clone()),
		odometer: Some(odometer),
		earned_custom_color: Some(false),
		retired: false,
		vs_result: Some(wm::save_game_result_request::VersusBattleResult {
			result: 0,
			survived: true,
			// Bayshore appears to not care
			num_of_players: 0,
			area: 0,
			is_morning: false,
			vs_play_count,
			vs_cool_or_wild: Some(vs_cool_or_wild),
			vs_smooth_or_rough: Some(vs_smooth_or_rough),

			..Default::default()
		}),
		..Default::default()
	};

	wm::send_request(req, server, "method/save_game_result").await
}
