use crate::*;
use anyhow::Result;
use eframe::egui;
use url::Url;

pub struct UserItems {
	pub selected_category: Option<wm::ItemCategory>,
	pub new_item_buf: Option<u32>,
}

fn show_tickets(car_items: &Vec<&wm::UserItem>, ui: &mut egui::Ui, new_item_buf: &mut Option<u32>) {
	let mut ft = 0;
	let mut discarded = 0;

	for item in car_items {
		if item.item_id == 5 {
			ft = ft + 1;
		} else if item.item_id == 1 || item.item_id == 2 || item.item_id == 3 {
			discarded = discarded + 1;
		}
	}
	if ft > 0 {
		ui.label(format!("Full Tune Tickets: {ft}",));
	}
	if discarded > 0 {
		ui.label(format!("Discarded Vehicle Tickets: {discarded}"));
	}

	let selected = match new_item_buf {
		Some(id) => match id {
			1 => String::from("Discarded Vehicle Ticket"),
			5 => String::from("Full Tune Ticket"),
			_ => String::from("This Message Should Not Appear."),
		},
		None => String::new(),
	};

	egui::ComboBox::from_id_source("EnumComboBox")
		.selected_text(selected)
		.wrap(true)
		.show_ui(ui, |ui| {
			ui.selectable_value(new_item_buf, Some(5), "Full Tune Ticket");
			ui.selectable_value(new_item_buf, Some(1), "Discarded Vehicle Ticket");
		});
}

fn wait_update_items(
	car_items: &[wm::UserItem],
	category: wm::ItemCategory,
	ui: &mut egui::Ui,
	new_item_buf: &mut Option<u32>,
) {
	let mut car_items = car_items
		.iter()
		.filter(|i| i.category == category.into())
		.collect::<Vec<_>>();
	car_items.sort_by(|a, b| a.item_id.cmp(&b.item_id));
	if category == wm::ItemCategory::CatCarTicketFree {
		show_tickets(&car_items, ui, new_item_buf);
	}
}

fn wait_select_category(ui: &mut egui::Ui) -> Option<wm::ItemCategory> {
	if ui.button("Tickets").clicked() {
		Some(wm::ItemCategory::CatCarTicketFree)
	} else {
		None
	}
}

async fn update_user_items(
	server: &Url,
	car: wm::Car,
	setting: Option<wm::CarSetting>,
	earned_user_items: Vec<wm::UserItem>,
	odometer: u32,
) -> Result<wm::UpdateCarResponse> {
	let req = wm::SaveGameResultRequest {
		car_id: car.car_id(),
		game_mode: wm::GameMode::ModeEvent as i32,
		played_at: 0,
		play_count: 0,
		retired: true,
		timeup: true,
		no_credit: Some(true),
		car: Some(car),
		setting,
		odometer: Some(odometer),
		earned_custom_color: Some(false),
		confirmed_tutorials: vec![],
		earned_items: vec![],
		earned_user_items,
		preserved_titles: vec![],
		neighbor_cars: vec![],
		st_result: None,
		ta_result: None,
		vs_result: None,
		rg_result: None,
		koshien_last_played_state: None,
	};

	wm::send_request(req, server, "method/save_game_result").await
}

impl UserItems {
	pub fn update(
		&mut self,
		ui: &mut egui::Ui,
		runtime: &tokio::runtime::Runtime,
		server: &Url,
		items: &mut Vec<wm::UserItem>,
		car: &wm::Car,
		odometer: u32,
	) {
		runtime.block_on(async {
			if self.selected_category.is_none() {
				self.selected_category = wait_select_category(ui);
			} else if let Some(selected_category) = self.selected_category {
				wait_update_items(items, selected_category, ui, &mut self.new_item_buf);
				if ui.button("Add item").clicked()
					&& let Some(item_id) = self.new_item_buf
				{
					let item = wm::UserItem {
						category: selected_category.into(),
						item_id,
						user_item_id: None,
						earned_at: None,
						expire_at: None,
						title_name: None,
					};
					if update_user_items(server, car.clone(), None, vec![item.clone()], odometer)
						.await
						.is_ok()
					{
						items.push(item);
						self.new_item_buf = None;
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
