<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from "svelte";

  let content = "";
  let feedback = "";
  let version = 0;
  let isLoading = false;

  let timeoutId: number;

  async function analyzeContent() {
    isLoading = true;
    try {
      const response = await invoke<{ content: string; version: number }>("analyze_content", { content });
      feedback = response.content;
      version = response.version;
    } catch (error) {
      console.error("Error analyzing content:", error);
    } finally {
      isLoading = false;
    }
  }

  function handleInput() {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(analyzeContent, 1000); // Debounce for 1 second
  }

  onMount(() => {
    return () => clearTimeout(timeoutId);
  });
</script>

<main class="container">
  <div class="editor-container">
    <textarea
      bind:value={content}
      on:input={handleInput}
      placeholder="Start writing your article..."
    />
  </div>
  
  <div class="feedback-container">
    <h2>Writing Feedback (Version {version})</h2>
    {#if isLoading}
      <div class="loading">Analyzing your writing...</div>
    {:else}
      <div class="feedback">{feedback}</div>
    {/if}
  </div>
</main>

<style>
  .container {
    display: flex;
    height: 100vh;
    padding: 1rem;
    gap: 1rem;
  }

  .editor-container {
    flex: 1;
  }

  .feedback-container {
    flex: 1;
    padding: 1rem;
    background-color: #f5f5f5;
    border-radius: 4px;
  }

  textarea {
    width: 100%;
    height: 100%;
    padding: 1rem;
    border: 1px solid #ccc;
    border-radius: 4px;
    resize: none;
  }

  .loading {
    color: #666;
    font-style: italic;
  }

  .feedback {
    white-space: pre-wrap;
  }
</style>