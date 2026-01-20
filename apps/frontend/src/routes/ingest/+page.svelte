<script lang="ts">
  import { api } from '$lib/api/client';

  type Tab = 'text' | 'url';

  let activeTab = $state<Tab>('text');
  let textContent = $state('');
  let textTitle = $state('');
  let urlInput = $state('');
  let maxCards = $state(10);
  let loading = $state(false);
  let error = $state<string | null>(null);

  interface StagedCard {
    temp_id: string;
    front: string;
    back: string;
    tags: string[];
    approved: boolean;
  }

  let sourceId = $state('');
  let stagedCards = $state<StagedCard[]>([]);
  let showPreview = $state(false);

  async function handleTextSubmit() {
    if (!textContent.trim()) return;

    loading = true;
    error = null;

    try {
      const response = await fetch('/api/ingest/text', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          content: textContent,
          title: textTitle || undefined,
          max_cards: maxCards
        })
      });

      const data = await response.json();

      if (data.error) {
        error = data.error;
      } else {
        sourceId = data.source_id;
        stagedCards = data.staged_cards;
        showPreview = true;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to generate cards';
    } finally {
      loading = false;
    }
  }

  async function handleUrlSubmit() {
    if (!urlInput.trim()) return;

    loading = true;
    error = null;

    try {
      const response = await fetch('/api/ingest/url', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          url: urlInput,
          max_cards: maxCards
        })
      });

      const data = await response.json();

      if (data.error) {
        error = data.error;
      } else {
        sourceId = data.source_id;
        stagedCards = data.staged_cards;
        showPreview = true;
      }
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to generate cards';
    } finally {
      loading = false;
    }
  }

  function toggleCard(tempId: string) {
    stagedCards = stagedCards.map(card =>
      card.temp_id === tempId ? { ...card, approved: !card.approved } : card
    );
  }

  function selectAll() {
    stagedCards = stagedCards.map(card => ({ ...card, approved: true }));
  }

  function deselectAll() {
    stagedCards = stagedCards.map(card => ({ ...card, approved: false }));
  }

  async function confirmCards() {
    loading = true;
    error = null;

    try {
      const response = await fetch('/api/ingest/confirm', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          source_id: sourceId,
          cards: stagedCards
        })
      });

      const data = await response.json();

      // Reset state
      showPreview = false;
      stagedCards = [];
      textContent = '';
      textTitle = '';
      urlInput = '';

      // Show success
      alert(`Successfully created ${data.created_count} cards!`);
    } catch (e) {
      error = e instanceof Error ? e.message : 'Failed to create cards';
    } finally {
      loading = false;
    }
  }

  function cancelPreview() {
    showPreview = false;
    stagedCards = [];
    sourceId = '';
  }

  $effect(() => {
    const approvedCount = stagedCards.filter(c => c.approved).length;
  });
</script>

<svelte:head>
  <title>Import Content - Engram</title>
</svelte:head>

<div class="ingest-page">
  <h1>Import Content</h1>
  <p class="subtitle">Generate flashcards from text or URLs using AI</p>

  {#if !showPreview}
    <div class="tabs">
      <button
        class="tab"
        class:active={activeTab === 'text'}
        onclick={() => (activeTab = 'text')}
      >
        Text
      </button>
      <button
        class="tab"
        class:active={activeTab === 'url'}
        onclick={() => (activeTab = 'url')}
      >
        URL
      </button>
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    <div class="form-container">
      {#if activeTab === 'text'}
        <div class="form-group">
          <label for="title">Title (optional)</label>
          <input
            type="text"
            id="title"
            bind:value={textTitle}
            placeholder="e.g., Biology Chapter 5"
          />
        </div>
        <div class="form-group">
          <label for="content">Content</label>
          <textarea
            id="content"
            bind:value={textContent}
            rows="12"
            placeholder="Paste your text content here..."
          ></textarea>
        </div>
      {:else}
        <div class="form-group">
          <label for="url">URL</label>
          <input
            type="url"
            id="url"
            bind:value={urlInput}
            placeholder="https://example.com/article"
          />
        </div>
      {/if}

      <div class="form-group">
        <label for="maxCards">Maximum Cards</label>
        <input
          type="number"
          id="maxCards"
          bind:value={maxCards}
          min="1"
          max="50"
        />
      </div>

      <button
        class="btn btn-primary"
        disabled={loading || (activeTab === 'text' ? !textContent.trim() : !urlInput.trim())}
        onclick={activeTab === 'text' ? handleTextSubmit : handleUrlSubmit}
      >
        {loading ? 'Generating...' : 'Generate Cards'}
      </button>
    </div>
  {:else}
    <div class="preview">
      <div class="preview-header">
        <h2>Preview Generated Cards</h2>
        <div class="preview-actions">
          <button class="btn-link" onclick={selectAll}>Select All</button>
          <button class="btn-link" onclick={deselectAll}>Deselect All</button>
        </div>
      </div>

      <p class="preview-count">
        {stagedCards.filter(c => c.approved).length} of {stagedCards.length} cards selected
      </p>

      {#if error}
        <div class="error">{error}</div>
      {/if}

      <div class="card-list">
        {#each stagedCards as card (card.temp_id)}
          <div
            class="staged-card"
            class:approved={card.approved}
            onclick={() => toggleCard(card.temp_id)}
          >
            <div class="card-checkbox">
              <input
                type="checkbox"
                checked={card.approved}
                onclick={(e) => e.stopPropagation()}
                onchange={() => toggleCard(card.temp_id)}
              />
            </div>
            <div class="card-content">
              <div class="card-front">
                <span class="label">Front</span>
                <p>{card.front}</p>
              </div>
              <div class="card-back">
                <span class="label">Back</span>
                <p>{card.back}</p>
              </div>
              {#if card.tags.length > 0}
                <div class="card-tags">
                  {#each card.tags as tag}
                    <span class="tag">{tag}</span>
                  {/each}
                </div>
              {/if}
            </div>
          </div>
        {/each}
      </div>

      <div class="preview-footer">
        <button class="btn btn-secondary" onclick={cancelPreview}>
          Cancel
        </button>
        <button
          class="btn btn-primary"
          disabled={loading || stagedCards.filter(c => c.approved).length === 0}
          onclick={confirmCards}
        >
          {loading ? 'Creating...' : `Add ${stagedCards.filter(c => c.approved).length} Cards`}
        </button>
      </div>
    </div>
  {/if}
</div>

<style>
  .ingest-page {
    max-width: 800px;
    margin: 0 auto;
  }

  h1 {
    font-size: 1.75rem;
    margin-bottom: 0.5rem;
  }

  .subtitle {
    color: var(--text-secondary);
    margin-bottom: 2rem;
  }

  .tabs {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.5rem;
  }

  .tab {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text-secondary);
    cursor: pointer;
    transition: all 0.2s;
  }

  .tab.active {
    background: var(--primary);
    color: white;
    border-color: var(--primary);
  }

  .error {
    background: #fee2e2;
    color: var(--danger);
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }

  .form-container {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .form-group {
    margin-bottom: 1rem;
  }

  .form-group label {
    display: block;
    font-weight: 500;
    margin-bottom: 0.5rem;
  }

  .form-group input,
  .form-group textarea {
    width: 100%;
    padding: 0.75rem;
    font-size: 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--background);
    color: var(--text);
    font-family: inherit;
  }

  .form-group textarea {
    resize: vertical;
  }

  .form-group input[type='number'] {
    width: 120px;
  }

  .btn {
    padding: 0.75rem 1.5rem;
    font-size: 1rem;
    font-weight: 500;
    border-radius: 8px;
    border: none;
    cursor: pointer;
    transition: background 0.2s;
  }

  .btn:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-primary {
    background: var(--primary);
    color: white;
  }

  .btn-secondary {
    background: var(--surface);
    color: var(--text);
    border: 1px solid var(--border);
  }

  .btn-link {
    background: none;
    border: none;
    color: var(--primary);
    cursor: pointer;
    font-size: 0.875rem;
  }

  .preview {
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .preview-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
  }

  .preview-header h2 {
    font-size: 1.25rem;
  }

  .preview-actions {
    display: flex;
    gap: 1rem;
  }

  .preview-count {
    color: var(--text-secondary);
    margin-bottom: 1rem;
  }

  .card-list {
    max-height: 500px;
    overflow-y: auto;
    margin-bottom: 1rem;
  }

  .staged-card {
    display: flex;
    gap: 1rem;
    padding: 1rem;
    border: 1px solid var(--border);
    border-radius: 8px;
    margin-bottom: 0.75rem;
    cursor: pointer;
    opacity: 0.6;
    transition: opacity 0.2s;
  }

  .staged-card.approved {
    opacity: 1;
    border-color: var(--primary);
  }

  .card-checkbox {
    display: flex;
    align-items: flex-start;
    padding-top: 0.25rem;
  }

  .card-checkbox input {
    width: 20px;
    height: 20px;
    cursor: pointer;
  }

  .card-content {
    flex: 1;
  }

  .card-front,
  .card-back {
    margin-bottom: 0.5rem;
  }

  .label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
  }

  .card-front p,
  .card-back p {
    margin: 0.25rem 0 0;
  }

  .card-tags {
    display: flex;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-top: 0.5rem;
  }

  .tag {
    font-size: 0.75rem;
    padding: 0.25rem 0.5rem;
    background: var(--surface-alt);
    border-radius: 4px;
    color: var(--text-secondary);
  }

  .preview-footer {
    display: flex;
    gap: 1rem;
    justify-content: flex-end;
  }
</style>
