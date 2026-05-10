use super::*;
use codex_app_server_protocol::ThreadRealtimeAudioChunk;
use codex_app_server_protocol::ThreadRealtimeClosedNotification;
use codex_app_server_protocol::ThreadRealtimeErrorNotification;
use codex_app_server_protocol::ThreadRealtimeItemAddedNotification;
use codex_app_server_protocol::ThreadRealtimeOutputAudioDeltaNotification;
use codex_app_server_protocol::ThreadRealtimeStartedNotification;
#[cfg(not(target_os = "linux"))]
use std::sync::atomic::AtomicU16;
#[cfg(not(target_os = "linux"))]
use std::time::Duration;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) enum RealtimeConversationPhase {
    #[default]
    Inactive,
    Starting,
    Active,
    Stopping,
}

#[derive(Default)]
pub(super) struct RealtimeConversationUiState {
    pub(super) phase: RealtimeConversationPhase,
    requested_close: bool,
    realtime_session_id: Option<String>,
    #[cfg(not(target_os = "linux"))]
    pub(super) meter_placeholder_id: Option<String>,
    #[cfg(not(target_os = "linux"))]
    capture_stop_flag: Option<Arc<AtomicBool>>,
    #[cfg(not(target_os = "linux"))]
    capture: Option<crate::voice::VoiceCapture>,
    #[cfg(not(target_os = "linux"))]
    audio_player: Option<crate::voice::RealtimeAudioPlayer>,
}

impl RealtimeConversationUiState {
    pub(super) fn is_live(&self) -> bool {
        matches!(
            self.phase,
            RealtimeConversationPhase::Starting
                | RealtimeConversationPhase::Active
                | RealtimeConversationPhase::Stopping
        )
    }

    #[cfg(not(target_os = "linux"))]
    pub(super) fn is_active(&self) -> bool {
        matches!(self.phase, RealtimeConversationPhase::Active)
    }
}

impl ChatWidget {
    fn realtime_footer_hint_items() -> Vec<(String, String)> {
        vec![("/realtime".to_string(), "stop live voice".to_string())]
    }

    pub(super) fn stop_realtime_conversation_from_ui(&mut self) {
        self.request_realtime_conversation_close(/*info_message*/ None);
    }

    #[cfg(not(target_os = "linux"))]
    pub(crate) fn stop_realtime_conversation_for_deleted_meter(&mut self, id: &str) -> bool {
        if self.realtime_conversation.is_live()
            && self.realtime_conversation.meter_placeholder_id.as_deref() == Some(id)
        {
            self.realtime_conversation.meter_placeholder_id = None;
            self.stop_realtime_conversation_from_ui();
            return true;
        }

        false
    }

    pub(super) fn start_realtime_conversation(&mut self) {
        self.realtime_conversation.phase = RealtimeConversationPhase::Starting;
        self.realtime_conversation.requested_close = false;
        self.realtime_conversation.realtime_session_id = None;
        self.set_footer_hint_override(Some(Self::realtime_footer_hint_items()));
        self.submit_realtime_conversation_start();
        self.request_redraw();
    }

    fn submit_realtime_conversation_start(&mut self) {
        self.submit_op(AppCommand::realtime_conversation_start(
            None,
            self.config
                .realtime
                .voice
                .and_then(|voice| serde_json::to_value(voice).ok()),
        ));
    }

    pub(super) fn request_realtime_conversation_close(&mut self, info_message: Option<String>) {
        if !self.realtime_conversation.is_live() {
            if let Some(message) = info_message {
                self.add_info_message(message, /*hint*/ None);
            }
            return;
        }

        self.realtime_conversation.requested_close = true;
        self.realtime_conversation.phase = RealtimeConversationPhase::Stopping;
        self.submit_op(AppCommand::realtime_conversation_close());
        self.stop_realtime_local_audio();
        self.set_footer_hint_override(/*items*/ None);

        if let Some(message) = info_message {
            self.add_info_message(message, /*hint*/ None);
        } else {
            self.request_redraw();
        }
    }

    pub(super) fn reset_realtime_conversation_state(&mut self) {
        self.stop_realtime_local_audio();
        self.set_footer_hint_override(/*items*/ None);
        self.realtime_conversation.phase = RealtimeConversationPhase::Inactive;
        self.realtime_conversation.requested_close = false;
        self.realtime_conversation.realtime_session_id = None;
    }

    fn fail_realtime_conversation(&mut self, message: String) {
        self.add_error_message(message);
        if self.realtime_conversation.is_live() {
            self.request_realtime_conversation_close(/*info_message*/ None);
        } else {
            self.reset_realtime_conversation_state();
            self.request_redraw();
        }
    }

    pub(super) fn on_realtime_conversation_started(
        &mut self,
        notification: ThreadRealtimeStartedNotification,
    ) {
        if !self.realtime_conversation_enabled() {
            self.request_realtime_conversation_close(/*info_message*/ None);
            return;
        }
        self.realtime_conversation.realtime_session_id = notification.realtime_session_id;
        self.set_footer_hint_override(Some(Self::realtime_footer_hint_items()));
        self.realtime_conversation.phase = RealtimeConversationPhase::Active;
        self.start_realtime_local_audio();
        self.request_redraw();
    }

    pub(super) fn on_realtime_output_audio_delta(
        &mut self,
        notification: ThreadRealtimeOutputAudioDeltaNotification,
    ) {
        self.enqueue_realtime_audio_out(&notification.audio);
    }

    pub(super) fn on_realtime_item_added(
        &mut self,
        notification: ThreadRealtimeItemAddedNotification,
    ) {
        if matches!(
            notification
                .item
                .get("type")
                .and_then(|value| value.as_str()),
            Some("input_audio_buffer.speech_started" | "response.cancelled")
        ) {
            self.interrupt_realtime_audio_playback();
        }
    }

    pub(super) fn on_realtime_error(&mut self, notification: ThreadRealtimeErrorNotification) {
        self.fail_realtime_conversation(format!("Realtime voice error: {}", notification.message));
    }

    pub(super) fn on_realtime_conversation_closed(
        &mut self,
        notification: ThreadRealtimeClosedNotification,
    ) {
        let requested = self.realtime_conversation.requested_close;
        let reason = notification.reason;
        self.reset_realtime_conversation_state();
        if !requested
            && let Some(reason) = reason
            && reason != "error"
        {
            self.add_info_message(
                format!("Realtime voice mode closed: {reason}"),
                /*hint*/ None,
            );
        }
        self.request_redraw();
    }

    pub(super) fn on_realtime_conversation_sdp(&mut self, _sdp: String) {}

    fn enqueue_realtime_audio_out(&mut self, frame: &ThreadRealtimeAudioChunk) {
        #[cfg(not(target_os = "linux"))]
        {
            if self.realtime_conversation.audio_player.is_none() {
                self.realtime_conversation.audio_player =
                    crate::voice::RealtimeAudioPlayer::start(&self.config).ok();
            }
            if let Some(player) = &self.realtime_conversation.audio_player
                && let Err(err) = player.enqueue_frame(frame)
            {
                warn!("failed to play realtime audio: {err}");
            }
        }
        #[cfg(target_os = "linux")]
        {
            let _ = frame;
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn interrupt_realtime_audio_playback(&mut self) {
        if let Some(player) = &self.realtime_conversation.audio_player {
            player.clear();
        }
    }

    #[cfg(target_os = "linux")]
    fn interrupt_realtime_audio_playback(&mut self) {}

    #[cfg(not(target_os = "linux"))]
    fn start_realtime_local_audio(&mut self) {
        if self.realtime_conversation.capture_stop_flag.is_some() {
            return;
        }

        let capture = match crate::voice::VoiceCapture::start_realtime(
            &self.config,
            self.app_event_tx.clone(),
        ) {
            Ok(capture) => capture,
            Err(err) => {
                self.fail_realtime_conversation(format!(
                    "Failed to start microphone capture: {err}"
                ));
                return;
            }
        };

        let stop_flag = capture.stopped_flag();
        let peak = capture.last_peak_arc();
        self.start_realtime_meter(stop_flag.clone(), peak);
        self.realtime_conversation.capture_stop_flag = Some(stop_flag);
        self.realtime_conversation.capture = Some(capture);
        if self.realtime_conversation.audio_player.is_none() {
            self.realtime_conversation.audio_player =
                crate::voice::RealtimeAudioPlayer::start(&self.config).ok();
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn start_realtime_meter(&mut self, stop_flag: Arc<AtomicBool>, peak: Arc<AtomicU16>) {
        let placeholder_id = self.bottom_pane.insert_recording_meter_placeholder("⠤⠤⠤⠤");
        self.realtime_conversation.meter_placeholder_id = Some(placeholder_id.clone());
        self.request_redraw();

        start_realtime_meter_task(placeholder_id, self.app_event_tx.clone(), stop_flag, peak);
    }

    #[cfg(target_os = "linux")]
    fn start_realtime_local_audio(&mut self) {}

    #[cfg(not(target_os = "linux"))]
    pub(crate) fn restart_realtime_audio_device(&mut self, kind: RealtimeAudioDeviceKind) {
        if !self.realtime_conversation.is_active() {
            return;
        }

        match kind {
            RealtimeAudioDeviceKind::Microphone => {
                self.stop_realtime_microphone();
                self.start_realtime_local_audio();
            }
            RealtimeAudioDeviceKind::Speaker => {
                self.stop_realtime_speaker();
                match crate::voice::RealtimeAudioPlayer::start(&self.config) {
                    Ok(player) => {
                        self.realtime_conversation.audio_player = Some(player);
                    }
                    Err(err) => {
                        self.fail_realtime_conversation(format!(
                            "Failed to start speaker output: {err}"
                        ));
                    }
                }
            }
        }
        self.request_redraw();
    }

    #[cfg(target_os = "linux")]
    pub(crate) fn restart_realtime_audio_device(&mut self, kind: RealtimeAudioDeviceKind) {
        let _ = kind;
    }

    #[cfg(not(target_os = "linux"))]
    fn stop_realtime_local_audio(&mut self) {
        self.stop_realtime_microphone();
        self.stop_realtime_speaker();
    }

    #[cfg(target_os = "linux")]
    fn stop_realtime_local_audio(&mut self) {}

    #[cfg(not(target_os = "linux"))]
    fn stop_realtime_microphone(&mut self) {
        if let Some(flag) = self.realtime_conversation.capture_stop_flag.take() {
            flag.store(true, Ordering::Relaxed);
        }
        if let Some(capture) = self.realtime_conversation.capture.take() {
            capture.stop();
        }
        if let Some(id) = self.realtime_conversation.meter_placeholder_id.take() {
            self.remove_recording_meter_placeholder(&id);
        }
    }

    #[cfg(not(target_os = "linux"))]
    fn stop_realtime_speaker(&mut self) {
        if let Some(player) = self.realtime_conversation.audio_player.take() {
            player.clear();
        }
    }
}

#[cfg(not(target_os = "linux"))]
fn start_realtime_meter_task(
    meter_placeholder_id: String,
    app_event_tx: AppEventSender,
    stop_flag: Arc<AtomicBool>,
    peak: Arc<AtomicU16>,
) {
    std::thread::spawn(move || {
        let mut meter = crate::voice::RecordingMeterState::new();

        loop {
            if stop_flag.load(Ordering::Relaxed) {
                break;
            }

            let meter_text = meter.next_text(peak.load(Ordering::Relaxed));
            app_event_tx.send(AppEvent::UpdateRecordingMeter {
                id: meter_placeholder_id.clone(),
                text: meter_text,
            });

            std::thread::sleep(Duration::from_millis(60));
        }
    });
}
