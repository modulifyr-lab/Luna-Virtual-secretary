// Front-end bridge script for Luna

window.addEventListener('DOMContentLoaded', () => {
  const textInput = document.getElementById('text-input');
  const sendBtn = document.getElementById('send-btn');
  const messagesList = document.getElementById('messages');
  const thoughtLog = document.getElementById('thought-log');
  const sttStatusIndicator = document.getElementById('stt-status-indicator');
  const connectionStatusIndicator = document.getElementById('connection-status');

  // Tauri IPC invoke helper
  async function invokeCommand(cmd, payload = {}) {
    if (window.__TAURI__ && window.__TAURI__.tauri) {
      return await window.__TAURI__.tauri.invoke(cmd, payload);
    }
    return null;
  }

  async function updateConnectionStatus() {
    if (!connectionStatusIndicator) return;
    try {
      const statusText = await invokeCommand('get_status');
      if (statusText) {
        connectionStatusIndicator.textContent = statusText;
        if (statusText.includes('Online')) {
          connectionStatusIndicator.className = 'status-indicator online';
        } else {
          connectionStatusIndicator.className = 'status-indicator offline';
        }
      }
    } catch (e) {
      console.error('Failed to get connectivity status:', e);
    }
  }

  async function sendPrompt(promptText, senderLabel = 'User') {
    if (!promptText || !promptText.trim()) return;

    appendMessage(senderLabel, promptText, 'user');
    textInput.value = '';

    thoughtLog.innerHTML = '<p>Thinking...</p>';

    try {
      const res = await invokeCommand('process_user_input', { input: promptText });
      if (res) {
        appendMessage('Luna', res, 'assistant');
      } else {
        appendMessage('Luna', 'Luna did not return a response.', 'assistant');
      }
    } catch (err) {
      appendMessage('Luna', `Error: ${err}`, 'assistant');
    }
    updateConnectionStatus();
  }

  function appendMessage(sender, text, type) {
    const msgDiv = document.createElement('div');
    msgDiv.className = `message ${type}`;
    msgDiv.innerHTML = `<span class="sender">${sender}:</span> <span class="text">${text}</span>`;
    messagesList.appendChild(msgDiv);
    messagesList.scrollTop = messagesList.scrollHeight;
  }

  function updateSttState(state) {
    if (!sttStatusIndicator) return;
    sttStatusIndicator.className = 'status-indicator ' + state;

    if (state === 'listening') {
      sttStatusIndicator.textContent = 'STT: Listening...';
      thoughtLog.innerHTML = '<p class="listening-text">🎤 Listening to voice input...</p>';
    } else if (state === 'transcribing') {
      sttStatusIndicator.textContent = 'STT: Transcribing...';
      thoughtLog.innerHTML = '<p class="transcribing-text">⚡ Transcribing audio with Whisper...</p>';
    } else if (state === 'speaking') {
      sttStatusIndicator.textContent = 'TTS: Speaking...';
      thoughtLog.innerHTML = '<p class="speaking-text">🔊 Luna is speaking...</p>';
    } else {
      sttStatusIndicator.textContent = 'STT: Idle';
      thoughtLog.innerHTML = '<p class="idle-text">Luna is ready.</p>';
    }
  }

  sendBtn.addEventListener('click', () => {
    sendPrompt(textInput.value);
  });

  textInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      sendPrompt(textInput.value);
    }
  });

  // Initial status check and periodic updates
  updateConnectionStatus();
  setInterval(updateConnectionStatus, 15000);

  // Listen to Tauri backend events for hotkey STT pipeline
  if (window.__TAURI__ && window.__TAURI__.event) {
    window.__TAURI__.event.listen('stt-state-changed', (event) => {
      const state = event.payload;
      updateSttState(state);
    });

    window.__TAURI__.event.listen('stt-transcribed-text', (event) => {
      const transcribedText = event.payload;
      if (transcribedText && !transcribedText.startsWith('[Error:')) {
        sendPrompt(transcribedText, 'User (Voice)');
      } else if (transcribedText) {
        appendMessage('System', transcribedText, 'assistant');
      }
    });
  }
});
