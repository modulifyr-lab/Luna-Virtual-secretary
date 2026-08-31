// Front-end bridge script for Luna

window.addEventListener('DOMContentLoaded', () => {
  const textInput = document.getElementById('text-input');
  const sendBtn = document.getElementById('send-btn');
  const messagesList = document.getElementById('messages');
  const thoughtLog = document.getElementById('thought-log');

  // TODO: Setup Tauri IPC invoke wrapper when Tauri API is loaded
  async function sendPrompt(promptText) {
    if (!promptText.trim()) return;

    // Append user message stub
    appendMessage('User', promptText, 'user');
    textInput.value = '';

    // TODO: Invoke Tauri command 'process_user_input'
    thoughtLog.innerHTML = '<p>Thinking...</p>';

    // Temporary mock response
    setTimeout(() => {
      appendMessage('Luna', 'This is a stub response from Luna.', 'assistant');
      thoughtLog.innerHTML = '<p class="idle-text">Luna is ready.</p>';
    }, 500);
  }

  function appendMessage(sender, text, type) {
    const msgDiv = document.createElement('div');
    msgDiv.className = `message ${type}`;
    msgDiv.innerHTML = `<span class="sender">${sender}:</span> <span class="text">${text}</span>`;
    messagesList.appendChild(msgDiv);
    messagesList.scrollTop = messagesList.scrollHeight;
  }

  sendBtn.addEventListener('click', () => {
    sendPrompt(textInput.value);
  });

  textInput.addEventListener('keypress', (e) => {
    if (e.key === 'Enter') {
      sendPrompt(textInput.value);
    }
  });

  // TODO: Listen to Tauri backend events (e.g., hotkey-triggered speech transcription, state updates)
});
