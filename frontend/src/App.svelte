<script>
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { fade, slide } from 'svelte/transition';
  import { open } from '@tauri-apps/plugin-dialog';

  let filePath = '';
  let feedback = '';
  let monitoring = false;
  let error = '';
  let isLoading = false;
  $: version = '1';

  async function pickFile() {
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Text',
          extensions: ['txt', 'md']
        }]
      });
      if (selected) {
        filePath = selected;
      }
    } catch (err) {
      error = 'Error selecting file: ' + err;
    }
  }

  async function startMonitor() {
    try {
      error = '';
      await invoke('start_feedback_monitor', { filePath });
      monitoring = true;
    } catch (err) {
      error = 'Error starting monitor: ' + err;
    }
  }

  async function stopMonitor() {
    try {
      error = '';
      await invoke('stop_feedback_monitor');
    } catch (err) {
      error = 'Error stopping monitor: ' + err;
    }
  }

  onMount(() => {
    const unlistenPromises = [
      listen('feedback-update', event => {
        error = '';
        version = event.payload.version ? event.payload.version : version;
        feedback = `${event.payload.feedback}`;
      }),
      listen('monitoring-stopped', () => {
        monitoring = false;
        feedback = '';
      }),
      listen('loading-state', event => {
        isLoading = event.payload;
      })
    ];

    return () => {
      unlistenPromises.forEach(promise => promise.then(unlisten => unlisten()));
    };
  });
</script>

<style>
  :global(body) {
    margin: 0;
    background: linear-gradient(135deg, #f5f7fa 0%, #c3cfe2 100%);
    min-height: 100vh;
  }

  main {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: flex-start;
    min-height: 100vh;
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
    padding: 40px 20px;
    max-width: 800px;
    margin: 0 auto;
  }

  h1 {
    color: #2d3748;
    font-size: 2.5rem;
    margin-bottom: 2rem;
    text-align: center;
    font-weight: 700;
  }

  .input-container {
    width: 100%;
    max-width: 600px;
    background: white;
    padding: 2rem;
    border-radius: 12px;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .file-picker {
    display: flex;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .pick-button {
    flex: 0 0 auto;
    width: auto;
    white-space: nowrap;
  }

  input[readonly] {
    background-color: #f7fafc;
    cursor: default;
  }

  input {
    width: 100%;
    margin-bottom: 1rem;
    padding: 0.75rem 1rem;
    font-size: 1rem;
    border: 2px solid #e2e8f0;
    border-radius: 8px;
    transition: all 0.2s ease;
  }

  input:focus {
    outline: none;
    border-color: #4299e1;
    box-shadow: 0 0 0 3px rgba(66, 153, 225, 0.2);
  }

  button {
    width: 100%;
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 600;
    color: white;
    background: #4299e1;
    border: none;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  button:hover:not(:disabled) {
    background: #3182ce;
    transform: translateY(-1px);
  }

  button:disabled {
    background: #a0aec0;
    cursor: not-allowed;
  }

  .feedback, .error {
    width: 100%;
    max-width: 600px;
    margin-top: 1.5rem;
    font-size: 1rem;
    padding: 1.5rem;
    border-radius: 12px;
    line-height: 1.5;
  }

  .feedback {
    background: white;
    border-left: 4px solid #48bb78;
    box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
  }

  .error {
    background: #fff5f5;
    border-left: 4px solid #f56565;
    color: #c53030;
  }

  .status {
    margin-top: 1rem;
    font-size: 0.875rem;
    color: #718096;
  }

  .monitoring {
    color: #48bb78;
    font-weight: 600;
  }

  .spinner {
    display: inline-block;
    width: 24px;
    height: 24px;
    border: 3px solid transparent;
    border-radius: 50%;
    border-top-color: #4299e1;
    border-right-color: #4299e1;
    animation: spin 0.8s linear infinite;
    margin-left: 8px;
  }

  @keyframes spin {
    from {
      transform: rotate(0deg);
    }
    to {
      transform: rotate(360deg);
    }
  }

  .feedback-container {
    width: 100%;
    max-width: 600px;
    margin-top: 1.5rem;
    position: relative;
  }

  .feedback-header {
    display: flex;
    align-items: center;
    margin-bottom: 0.5rem;
  }
</style>

<main>
  <h1>Realtime Article Feedback</h1>
  
  <div class="input-container" transition:fade>
    <div class="file-picker">
      <input
        type="text"
        value={filePath}
        placeholder="Select your article file"
        disabled={true}
        readonly
      />
      <button class="pick-button" on:click={pickFile} disabled={monitoring}>
        Choose File
      </button>
    </div>
    <button on:click={monitoring ? stopMonitor : startMonitor} disabled={!filePath}>
      {monitoring ? 'Stop Monitoring' : 'Start Monitoring'}
    </button>
    
    {#if monitoring}
      <p class="status">Status: <span class="monitoring">Active</span></p>
    {/if}
  </div>

  {#if error}
    <div class="error" transition:slide>
      {error}
    </div>
  {/if}

  {#if feedback || isLoading}
    <div class="feedback-container">
      <div class="feedback-header">
        <h3>Version {version}&nbsp;</h3>
        {#if isLoading}
          <div class="spinner"></div>
        {/if}
      </div>
      {#if feedback}
        <div class="feedback" transition:slide>
          {feedback}
        </div>
      {/if}
    </div>
  {/if}
</main> 