use crate::*;
use anyhow::Result;
use eframe::egui;
use num_traits::{FromPrimitive, ToPrimitive};
use strum::IntoEnumIterator;
use url::Url;

fn show_enum_items<T>(car_items: &Vec<&wm::CarItem>, ui: &mut egui::Ui, new_item_buf: &mut String)
where
	T: FromPrimitive + ToPrimitive,
	T: ToString,
	T: IntoEnumIterator,
{
	if !car_items.is_empty() {
		ui.heading("Current items");
		for item in car_items.iter() {
			if let Some(item) = T::from_u32(item.item_id) {
				ui.label(item.to_string());
			}
		}
	}
	let selected = match new_item_buf.parse() {
		Ok(id) => match T::from_u32(id) {
			Some(item) => item.to_string(),
			None => String::new(),
		},
		Err(_) => String::new(),
	};

	egui::ComboBox::from_id_source("EnumComboBox")
		.selected_text(selected)
		.wrap(true)
		.show_ui(ui, |ui| {
			for id in T::iter() {
				if car_items
					.iter()
					.filter(|item| Some(item.item_id) == id.to_u32())
					.collect::<Vec<_>>()
					.is_empty()
				{
					ui.selectable_value(
						new_item_buf,
						id.to_u32().unwrap().to_string(),
						id.to_string(),
					);
				}
			}
		});
}

async fn give_all_enum_items<T>(server: &Url, car: &wm::Car, car_items: &mut Vec<wm::CarItem>)
where
	T: IntoEnumIterator,
	T: ToPrimitive,
	T: wm::GetCategory,
{
	let mut items = T::iter()
		.map(|id| wm::CarItem {
			item_id: id.to_u32().unwrap(),
			category: T::category().into(),
			amount: Some(1),
		})
		.filter(|item| !car_items.contains(item))
		.collect::<Vec<_>>();
	if update_car_items(server, car.clone(), None, items.clone())
		.await
		.is_ok()
	{
		car_items.append(&mut items);
	}
}

fn show_vs_items(car_items: &Vec<&wm::CarItem>, ui: &mut egui::Ui, new_item_buf: &mut String) {
	fn get_vs_aura(id: usize) -> String {
		if id > wm::VS_GRADES.len() * 3 {
			match id {
				100 => String::from("Anniversary"),
				101 => String::from("Halloween"),
				_ => String::from("Invalid aura"),
			}
		} else {
			format!("{} {}", wm::VS_GRADES[(id - 1) / 3], ((id - 1) % 3) + 1)
		}
	}

	if !car_items.is_empty() {
		ui.heading("Current items");
		for item in car_items.iter() {
			ui.label(get_vs_aura(item.item_id as usize));
		}
	}
	let selected = match new_item_buf.parse() {
		Ok(id) => get_vs_aura(id),
		Err(_) => String::new(),
	};
	egui::ComboBox::from_id_source("VsAuraComboBox")
		.selected_text(selected)
		.wrap(true)
		.show_ui(ui, |ui| {
			for grade in 1..=(wm::VS_GRADES.len() * 3) {
				if car_items
					.iter()
					.filter(|item| item.item_id == grade as u32)
					.collect::<Vec<_>>()
					.is_empty()
				{
					ui.selectable_value(new_item_buf, grade.to_string(), get_vs_aura(grade));
				}
			}
			for grade in 100..=101 {
				if car_items
					.iter()
					.filter(|item| item.item_id == grade as u32)
					.collect::<Vec<_>>()
					.is_empty()
				{
					ui.selectable_value(new_item_buf, grade.to_string(), get_vs_aura(grade));
				}
			}
		});
}

async fn give_all_vs_items(server: &Url, car: &wm::Car, car_items: &mut Vec<wm::CarItem>) {
	let mut items = vec![];
	for grade in 1..=(wm::VS_GRADES.len() * 3) {
		let item = wm::CarItem {
			item_id: grade as u32,
			category: wm::ItemCategory::CatAuraMotif.into(),
			amount: Some(1),
		};
		if !car_items.contains(&item) {
			items.push(item);
		}
	}
	for grade in 100..=101 {
		let item = wm::CarItem {
			item_id: grade as u32,
			category: wm::ItemCategory::CatAuraMotif.into(),
			amount: Some(1),
		};
		if !car_items.contains(&item) {
			items.push(item);
		}
	}
	if update_car_items(server, car.clone(), None, items.clone())
		.await
		.is_ok()
	{
		car_items.append(&mut items);
	}
}

async fn give_all_colors(server: &Url, car: &wm::Car, car_items: &mut Vec<wm::CarItem>) {
	let mut items = vec![];
	for color in 1..=40 {
		let item = wm::CarItem {
			item_id: color as u32,
			category: wm::ItemCategory::CatCustomColor.into(),
			amount: Some(1),
		};
		if !car_items.contains(&item) {
			items.push(item);
		}
	}
	if update_car_items(server, car.clone(), None, items.clone())
		.await
		.is_ok()
	{
		car_items.append(&mut items);
	}
}

async fn show_dress_up_items(
	car_items: &mut Vec<wm::CarItem>,
	ui: &mut egui::Ui,
	new_item_buf: &mut String,
	cars: &wm::Cars,
	server: &Url,
	car: &wm::Car,
) {
	fn category_to_str(category: wm::ItemCategory) -> &'static str {
		match category {
			wm::ItemCategory::CatWheel => "Wheels",
			wm::ItemCategory::CatAero => "Aero Set",
			wm::ItemCategory::CatBonnet => "Bonnet",
			wm::ItemCategory::CatWing => "Wing",
			wm::ItemCategory::CatMirror => "Mirror",
			wm::ItemCategory::CatNeon => "Neon Tube",
			wm::ItemCategory::CatTrunk => "Trunk",
			wm::ItemCategory::CatNumberPlate => "License Plate",
			wm::ItemCategory::CatGtWing => "GT Wing",
			wm::ItemCategory::CatAeroFullset => "Aero Set",
			wm::ItemCategory::CatAeroLimited => "Aero Set",
			_ => "Not a DU category?",
		}
	}

	// Not optimal at all
	let mut owned_items = car_items
		.iter()
		.filter_map(|i| {
			wm::DU_ITEMS
				.iter()
				.find(|j| j.server_id == i.item_id && j.category == i.category())
		})
		.filter(|i| cars.can_use_du_item(i))
		.collect::<Vec<_>>();
	owned_items.sort_by(|a, b| {
		if a.category.eq(&b.category) {
			a.name.cmp(b.name)
		} else {
			a.category.cmp(&b.category)
		}
	});
	let unowned_items = wm::DU_ITEMS
		.iter()
		.filter(|i| !owned_items.contains(i) && cars.can_use_du_item(i))
		.collect::<Vec<_>>();
	if !owned_items.is_empty() {
		let mut last_category = wm::ItemCategory::CatCustomColor;
		ui.heading("Current items");
		for item in owned_items {
			if item.category != last_category {
				last_category = item.category;
				ui.strong(category_to_str(item.category));
			}
			ui.label(item.name);
		}
	}

	egui::ComboBox::from_id_source("EnumComboBox")
		.selected_text(new_item_buf.to_string())
		.wrap(true)
		.show_ui(ui, |ui| {
			let mut last_category = wm::ItemCategory::CatCustomColor;
			for item in unowned_items {
				if item.category != last_category {
					last_category = item.category;
					ui.strong(category_to_str(item.category));
				}
				ui.selectable_value(new_item_buf, String::from(item.name), item.name);
			}
		});

	if ui.button("Add item").clicked() {
		let item = wm::DU_ITEMS
			.iter()
			.find(|item| new_item_buf == item.name && cars.can_use_du_item(item))
			.unwrap();
		if update_car_items(server, car.clone(), None, vec![item.clone().into()])
			.await
			.is_ok()
		{
			car_items.push(item.clone().into());
			new_item_buf.clear();
		}
	}
}

async fn wait_update_items(
	car_items: &mut Vec<wm::CarItem>,
	category: wm::ItemCategory,
	ui: &mut egui::Ui,
	new_item_buf: &mut String,
	cars: &wm::Cars,
	server: &Url,
	car: &wm::Car,
) {
	let mut new_car_items = car_items
		.iter()
		.filter(|i| i.category == category.into())
		.collect::<Vec<_>>();
	new_car_items.sort_by(|a, b| a.item_id.cmp(&b.item_id));
	if category == wm::ItemCategory::CatAuraMotif {
		show_vs_items(&new_car_items, ui, new_item_buf);
	} else if category == wm::ItemCategory::CatBgm {
		show_enum_items::<wm::Bgms>(&new_car_items, ui, new_item_buf);
	} else if category == wm::ItemCategory::CatMeter {
		show_enum_items::<wm::Meters>(&new_car_items, ui, new_item_buf);
	} else if category == wm::ItemCategory::CatNamePlate {
		show_enum_items::<wm::Nameplates>(&new_car_items, ui, new_item_buf);
	} else if category == wm::ItemCategory::CatRivalMarker {
		show_enum_items::<wm::RivalMarker>(&new_car_items, ui, new_item_buf);
	} else if category == wm::ItemCategory::CatCustomFrame {
		show_enum_items::<wm::CustomFrame>(&new_car_items, ui, new_item_buf);
	} else if category == wm::ItemCategory::CatAero {
		show_dress_up_items(car_items, ui, new_item_buf, cars, server, car).await;
	} else {
		if !car_items.is_empty() {
			ui.heading("Current items");
			for item in car_items.iter() {
				ui.label(format!("{}", item.item_id));
			}
		}
		ui.text_edit_singleline(new_item_buf);
	}
}

fn wait_select_category(ui: &mut egui::Ui) -> Option<wm::ItemCategory> {
	if ui.button("VS Aura").clicked() {
		Some(wm::ItemCategory::CatAuraMotif)
	} else if ui.button("Meter").clicked() {
		Some(wm::ItemCategory::CatMeter)
	} else if ui.button("BGM").clicked() {
		Some(wm::ItemCategory::CatBgm)
	} else if ui.button("Nameplate").clicked() {
		Some(wm::ItemCategory::CatNamePlate)
	} else if ui.button("Rival Marker").clicked() {
		Some(wm::ItemCategory::CatRivalMarker)
	} else if ui.button("Custom Frame").clicked() {
		Some(wm::ItemCategory::CatCustomFrame)
	} else if ui.button("Dress Up").clicked() {
		Some(wm::ItemCategory::CatAero)
	} else {
		None
	}
}

pub struct ItemMenu {
	pub new_item_buf: String,
	pub selected_category: Option<wm::ItemCategory>,
}

impl ItemMenu {
	pub fn update(
		&mut self,
		ui: &mut egui::Ui,
		runtime: &tokio::runtime::Runtime,
		server: &Url,
		car: &wm::Car,
		car_items: &mut Vec<wm::CarItem>,
	) {
		runtime.block_on(async {
			if self.selected_category.is_none() {
				self.selected_category = items::wait_select_category(ui);
				if car_items
					.iter()
					.filter(|i| i.category() == wm::ItemCategory::CatCustomColor)
					.collect::<Vec<_>>()
					.len() != 40 && ui.button("Give all custom colors").clicked()
				{
					give_all_colors(server, car, car_items).await;
				}
			} else if let Some(selected_category) = self.selected_category {
				items::wait_update_items(
					car_items,
					selected_category,
					ui,
					&mut self.new_item_buf,
					&wm::Cars::from_u32(car.visual_model()).unwrap(),
					server,
					car,
				)
				.await;

				if selected_category != wm::ItemCategory::CatAero && ui.button("Add item").clicked()
				{
					let item_id = match self.new_item_buf.parse() {
						Ok(item) => item,
						Err(_) => return,
					};
					let item = wm::CarItem {
						category: selected_category.into(),
						item_id,
						amount: Some(1),
					};
					if update_car_items(server, car.clone(), None, vec![item.clone()])
						.await
						.is_ok()
					{
						car_items.push(item);
						self.new_item_buf.clear();
					}
				}
				if ui.button("Give all").clicked() {
					if selected_category == wm::ItemCategory::CatAuraMotif {
						give_all_vs_items(server, car, car_items).await;
					} else if selected_category == wm::ItemCategory::CatBgm {
						give_all_enum_items::<wm::Bgms>(server, car, car_items).await;
					} else if selected_category == wm::ItemCategory::CatMeter {
						give_all_enum_items::<wm::Meters>(server, car, car_items).await;
					} else if selected_category == wm::ItemCategory::CatNamePlate {
						give_all_enum_items::<wm::Nameplates>(server, car, car_items).await;
					} else if selected_category == wm::ItemCategory::CatRivalMarker {
						give_all_enum_items::<wm::RivalMarker>(server, car, car_items).await;
					} else if selected_category == wm::ItemCategory::CatCustomFrame {
						give_all_enum_items::<wm::CustomFrame>(server, car, car_items).await;
					} else if selected_category == wm::ItemCategory::CatAero {
						let mut earned_items = vec![];
						let cars = wm::Cars::from_u32(car.visual_model()).unwrap();
						for item in wm::DU_ITEMS {
							if cars.can_use_du_item(&item) {
								earned_items.push(item.into());
							}
						}
						if update_car_items(server, car.clone(), None, earned_items.clone())
							.await
							.is_ok()
						{
							car_items.append(&mut earned_items);
						}
					}
				}
			}
		});
	}

	pub fn back(&mut self) -> bool {
		if self.selected_category.is_some() {
			self.selected_category = None;
			false
		} else {
			true
		}
	}
}

async fn update_car_items(
	server: &Url,
	car: wm::Car,
	setting: Option<wm::CarSetting>,
	earned_items: Vec<wm::CarItem>,
) -> Result<wm::UpdateCarResponse> {
	let req = wm::UpdateCarRequest {
		car_id: car.car_id(),
		car: Some(car),
		setting,
		earned_items,
		..Default::default()
	};

	wm::send_request(req, server, "method/update_car").await
}
