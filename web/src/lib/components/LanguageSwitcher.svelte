<script lang="ts">
  import { locale, t, availableLocales, type Locale } from '../i18n';

  function handleChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    locale.set(select.value as Locale);
  }
</script>

<div class="language-switcher">
  <label class="language-label" for="language-select">
    {$t.language}
  </label>
  <select
    id="language-select"
    class="language-select"
    value={$locale}
    onchange={handleChange}
  >
    {#each availableLocales as loc}
      <option value={loc.code}>{loc.nativeName}</option>
    {/each}
  </select>
</div>

<style>
  .language-switcher {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    margin-left: auto;
  }

  .language-label {
    font-size: 0.6875rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .language-select {
    padding: var(--space-1) var(--space-2);
    font-size: 0.8125rem;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .language-select:hover {
    border-color: var(--text-muted);
  }

  .language-select:focus {
    outline: none;
    border-color: var(--accent);
  }

  @media (max-width: 768px) {
    .language-label {
      display: none;
    }
  }
</style>
