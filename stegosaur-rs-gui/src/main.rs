use std::path::PathBuf;

use iced::widget::{button, column, container, image, row, scrollable, text, text_input};
use iced::{Element, Length, Task};

fn main() -> iced::Result {
    iced::application(State::default, update, view).run()
}

#[derive(Default)]
struct State {
    mode: Mode,
    source_path: Option<PathBuf>,
    encode_text: String,
    decoded_text: String,
    status_message: String,
}

#[derive(Debug, Clone, Default, PartialEq)]
enum Mode {
    #[default]
    Encode,
    Decode,
}

#[derive(Debug, Clone)]
enum Message {
    SwitchToEncode,
    SwitchToDecode,
    PickSourceFile,
    SourceFileSelected(Option<rfd::FileHandle>),
    EncodeTextChanged(String),
    EncodeAndSave,
    EncodeDone(Result<(), String>),
    Decode,
    DecodeDone(Result<String, String>),
}

fn update(state: &mut State, message: Message) -> Task<Message> {
    match message {
        Message::SwitchToEncode => {
            state.mode = Mode::Encode;
            state.status_message.clear();
            Task::none()
        }
        Message::SwitchToDecode => {
            state.mode = Mode::Decode;
            state.status_message.clear();
            state.decoded_text.clear();
            Task::none()
        }
        Message::PickSourceFile => Task::perform(
            rfd::AsyncFileDialog::new()
                .add_filter("PNG Image", &["png"])
                .set_title("Select PNG image")
                .pick_file(),
            Message::SourceFileSelected,
        ),
        Message::SourceFileSelected(handle) => {
            state.source_path = handle.map(|h| h.path().to_path_buf());
            state.status_message = state
                .source_path
                .as_ref()
                .map(|p| format!("Selected: {}", p.display()))
                .unwrap_or_default();
            Task::none()
        }
        Message::EncodeTextChanged(s) => {
            state.encode_text = s;
            Task::none()
        }
        Message::EncodeAndSave => {
            let source = match &state.source_path {
                Some(p) => p.clone(),
                None => {
                    state.status_message = "Please select an image first".to_string();
                    return Task::none();
                }
            };
            let text = state.encode_text.clone();
            if text.is_empty() {
                state.status_message = "Please enter text to encode".to_string();
                return Task::none();
            }
            Task::perform(
                async move {
                    let handle = rfd::AsyncFileDialog::new()
                        .add_filter("PNG Image", &["png"])
                        .set_title("Save encoded image as")
                        .set_file_name("encoded.png")
                        .save_file()
                        .await;
                    match handle {
                        Some(h) => {
                            let output_path = h.path().to_path_buf();
                            stegosaur_rs::encode_lossless(
                                source.to_str().unwrap(),
                                &text,
                                output_path.to_str().unwrap(),
                            )
                            .map_err(|e| e.to_string())
                        }
                        None => Err("Save cancelled".to_string()),
                    }
                },
                Message::EncodeDone,
            )
        }
        Message::EncodeDone(result) => {
            state.status_message = match result {
                Ok(()) => "Encoded successfully!".to_string(),
                Err(e) => format!("Encode failed: {}", e),
            };
            Task::none()
        }
        Message::Decode => {
            let source = match &state.source_path {
                Some(p) => p.clone(),
                None => {
                    state.status_message = "Please select an image first".to_string();
                    return Task::none();
                }
            };
            let path = source.to_path_buf();
            Task::perform(
                async move { stegosaur_rs::decode_lossless(path.to_str().unwrap()) },
                Message::DecodeDone,
            )
        }
        Message::DecodeDone(result) => {
            match &result {
                Ok(t) => {
                    state.decoded_text = t.clone();
                    state.status_message = "Decoded successfully".to_string();
                }
                Err(e) => {
                    state.decoded_text.clear();
                    state.status_message = format!("Decode failed: {}", e);
                }
            }
            Task::none()
        }
    }
}

fn view(state: &State) -> Element<'_, Message> {
    let mode_buttons = row![
        button("Encode")
            .on_press(Message::SwitchToEncode)
            .style(if state.mode == Mode::Encode {
                button::primary
            } else {
                button::secondary
            }),
        button("Decode")
            .on_press(Message::SwitchToDecode)
            .style(if state.mode == Mode::Decode {
                button::primary
            } else {
                button::secondary
            }),
    ]
    .spacing(10);

    let content: Element<Message> = match &state.mode {
        Mode::Encode => encode_view(state),
        Mode::Decode => decode_view(state),
    };

    let status = if state.status_message.is_empty() {
        Element::from(text(""))
    } else {
        Element::from(
            container(text(&state.status_message))
                .padding(8)
                .style(container::rounded_box),
        )
    };

    column![
        mode_buttons,
        content,
        status,
    ]
    .spacing(20)
    .padding(20)
    .into()
}

fn encode_view(state: &State) -> Element<'_, Message> {
    let pick_btn = button("Select Image").on_press(Message::PickSourceFile);

    let path_display = state
        .source_path
        .as_ref()
        .map(|p| text(p.display().to_string()).size(12))
        .unwrap_or_else(|| text("No image selected").size(12));

    let image_preview: Element<Message> = state
        .source_path
        .as_ref()
        .map(|p| {
            image(p.as_path())
                .width(Length::Fill)
                .height(200)
                .content_fit(iced::ContentFit::Contain)
                .into()
        })
        .unwrap_or_else(|| {
            container(text("Preview will appear here"))
                .width(Length::Fill)
                .height(200)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        });

    let text_input = text_input("Enter secret message...", &state.encode_text)
        .on_input(Message::EncodeTextChanged)
        .padding(8)
        .size(16);

    let encode_btn = button("Encode & Save")
        .on_press(Message::EncodeAndSave)
        .style(button::primary);

    column![
        pick_btn,
        path_display,
        image_preview,
        text_input,
        encode_btn,
    ]
    .spacing(15)
    .into()
}

fn decode_view(state: &State) -> Element<'_, Message> {
    let pick_btn = button("Select Image").on_press(Message::PickSourceFile);

    let path_display = state
        .source_path
        .as_ref()
        .map(|p| text(p.display().to_string()).size(12))
        .unwrap_or_else(|| text("No image selected").size(12));

    let image_preview: Element<Message> = state
        .source_path
        .as_ref()
        .map(|p| {
            image(p.as_path())
                .width(Length::Fill)
                .height(200)
                .content_fit(iced::ContentFit::Contain)
                .into()
        })
        .unwrap_or_else(|| {
            container(text("Preview will appear here"))
                .width(Length::Fill)
                .height(200)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
                .into()
        });

    let decode_btn = button("Decode")
        .on_press(Message::Decode)
        .style(button::primary);

    let decoded_display: Element<'_, Message> = if state.decoded_text.is_empty() {
        container(text("Decoded text will appear here"))
            .width(Length::Fill)
            .height(100)
            .padding(8)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .style(container::rounded_box)
            .into()
    } else {
        scrollable(
            container(text(&state.decoded_text))
                .width(Length::Fill)
                .padding(12)
                .style(container::rounded_box),
        )
        .height(150)
        .into()
    };

    column![
        pick_btn,
        path_display,
        image_preview,
        decode_btn,
        decoded_display,
    ]
    .spacing(15)
    .into()
}
