<script lang="ts">
  import { tileTheme, availableTileThemes, type TileTheme } from '../stores/tileTheme';
  import { t } from '../i18n';

  function handleChange(event: Event) {
    const select = event.target as HTMLSelectElement;
    tileTheme.set(select.value as TileTheme);
  }
</script>

<div class="tile-theme-switcher">
  <label class="tile-theme-label" for="tile-theme-select">
    {$t.tileTheme}
  </label>
  <select
    id="tile-theme-select"
    class="tile-theme-select"
    value={$tileTheme}
    onchange={handleChange}
  >
    {#each availableTileThemes as theme}
      <option value={theme.code}>
        {theme.code === 'light' ? $t.tileThemeLight : $t.tileThemeDark}
      </option>
    {/each}
  </select>
</div>

<style>
  .tile-theme-switcher {
    display: flex;
    align-items: center;
    gap: var(--space-2);
  }

  .tile-theme-label {
    font-size: 0.6875rem;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
  }

  .tile-theme-select {
    padding: var(--space-1) var(--space-2);
    font-size: 0.8125rem;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .tile-theme-select:hover {
    border-color: var(--text-muted);
  }

  .tile-theme-select:focus {
    outline: none;
    border-color: var(--accent);
  }

  @media (max-width: 768px) {
    .tile-theme-label {
      display: none;
    }
  }
</style>
