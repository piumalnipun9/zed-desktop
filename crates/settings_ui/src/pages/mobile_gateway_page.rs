use gpui::{ReadGlobal as _, ScrollHandle, prelude::*};
use settings::{MobileGatewayPairedDeviceSettingsContent, SettingsStore};
use ui::{Divider, prelude::*};

use crate::SettingsWindow;

pub(crate) fn render_paired_devices_page(
    _settings_window: &SettingsWindow,
    scroll_handle: &ScrollHandle,
    _window: &mut Window,
    cx: &mut Context<SettingsWindow>,
) -> AnyElement {
    // Read on every render, unlike the pages in `page_data`, so a removal shows as soon as the
    // settings file is written instead of after the settings window is reopened.
    let paired_devices = SettingsStore::global(cx)
        .raw_user_settings()
        .and_then(|settings| settings.content.mobile_gateway.as_ref())
        .and_then(|gateway| gateway.paired_devices.clone())
        .unwrap_or_default();

    let device_list = if paired_devices.is_empty() {
        render_empty_state(cx)
    } else {
        v_flex()
            .w_full()
            .gap_1()
            .children(itertools::intersperse_with(
                paired_devices
                    .iter()
                    .map(|device| render_paired_device(device).into_any_element()),
                || Divider::horizontal_dashed().into_any_element(),
            ))
            .into_any_element()
    };

    v_flex()
        .id("mobile-gateway-paired-devices-page")
        .track_scroll(scroll_handle)
        .size_full()
        .pt_2p5()
        .pb_16()
        .overflow_y_scroll()
        .child(
            v_flex()
                .w_full()
                .px_8()
                .gap_2()
                .child(
                    v_flex().child(Label::new("Paired Devices")).child(
                        Label::new(
                            "Phones that can reach this gateway. Removing one revokes its access immediately.",
                        )
                        .size(LabelSize::Small)
                        .color(Color::Muted),
                    ),
                )
                .child(device_list),
        )
        .into_any_element()
}

fn render_empty_state(cx: &App) -> AnyElement {
    h_flex()
        .p_4()
        .justify_center()
        .border_1()
        .border_dashed()
        .border_color(cx.theme().colors().border.opacity(0.6))
        .rounded_sm()
        .child(
            Label::new("No paired devices yet. Use \"Connect mobile device\" to pair a phone.")
                .color(Color::Muted)
                .size(LabelSize::Small),
        )
        .into_any_element()
}

fn render_paired_device(device: &MobileGatewayPairedDeviceSettingsContent) -> impl IntoElement {
    let device_id = device.id.clone();
    let paired_on = device
        .created_at
        .split('T')
        .next()
        .unwrap_or(&device.created_at)
        .to_string();

    h_flex()
        .w_full()
        .py_2()
        .gap_2()
        .justify_between()
        .child(
            h_flex()
                .min_w_0()
                .gap_2()
                .child(
                    Icon::new(IconName::Screen)
                        .size(IconSize::Small)
                        .color(Color::Muted),
                )
                .child(
                    v_flex()
                        .min_w_0()
                        .child(Label::new(device.label.clone()).truncate())
                        .child(
                            Label::new(format!("Paired {paired_on}"))
                                .size(LabelSize::Small)
                                .color(Color::Muted),
                        ),
                ),
        )
        .child(
            Button::new(SharedString::from(format!("remove-{device_id}")), "Remove")
                .style(ButtonStyle::OutlinedGhost)
                .size(ButtonSize::Medium)
                .on_click(move |_, _window, cx| {
                    let device_id = device_id.clone();
                    SettingsStore::global(cx).update_settings_file(
                        <dyn fs::Fs>::global(cx),
                        move |settings, _| {
                            if let Some(devices) = settings
                                .mobile_gateway
                                .as_mut()
                                .and_then(|gateway| gateway.paired_devices.as_mut())
                            {
                                devices.retain(|device| device.id != device_id);
                            }
                        },
                    );
                }),
        )
}
