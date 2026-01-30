<script lang="ts">
  import type { ScoringOutput } from '../agari';

  interface Props {
    result: ScoringOutput | null;
    error?: string | null;
    loading?: boolean;
  }

  let { result, error = null, loading = false }: Props = $props();

  // Format payment string
  const formatPayment = (payment: ScoringOutput['payment'], isDealer: boolean, isTsumo: boolean): string => {
    if (isTsumo) {
      if (isDealer) {
        return `${payment.from_non_dealer?.toLocaleString()} all`;
      } else {
        return `${payment.from_non_dealer?.toLocaleString()} / ${payment.from_dealer?.toLocaleString()}`;
      }
    } else {
      return `${payment.from_discarder?.toLocaleString()}`;
    }
  };

  // Get score level class
  const getScoreLevelClass = (level: string): string => {
    const normalized = level.toLowerCase().replace(/\s+/g, '');
    if (normalized === 'mangan') return 'mangan';
    if (normalized === 'haneman') return 'haneman';
    if (normalized === 'baiman') return 'baiman';
    if (normalized === 'sanbaiman') return 'sanbaiman';
    if (normalized.includes('yakuman')) return 'yakuman';
    return '';
  };

  const isTsumo = $derived(result?.payment.from_discarder === null || result?.payment.from_discarder === undefined);
</script>

<div class="score-result">
  {#if loading}
    <div class="loading">
      <div class="spinner"></div>
      <span>Calculating...</span>
    </div>
  {:else if error}
    <div class="error">
      <span class="error-icon">⚠️</span>
      <span class="error-message">{error}</span>
    </div>
  {:else if result}
    <div class="result-content">
      <!-- Score Summary -->
      <div class="score-summary">
        {#if result.score_level}
          <div class="score-level {getScoreLevelClass(result.score_level)}">
            {result.score_level}
          </div>
        {/if}

        <div class="score-numbers">
          <div class="han-fu">
            <span class="value">{result.total_han}</span>
            <span class="label">Han</span>
          </div>
          <div class="divider">/</div>
          <div class="han-fu">
            <span class="value">{result.fu}</span>
            <span class="label">Fu</span>
          </div>
        </div>

        <div class="total-points">
          <span class="points-value">{result.payment.total.toLocaleString()}</span>
          <span class="points-label">points</span>
        </div>

        <div class="payment-breakdown">
          {formatPayment(result.payment, result.is_dealer, isTsumo)}
          {#if result.is_dealer}
            <span class="dealer-badge">Dealer</span>
          {/if}
        </div>
      </div>

      <!-- Yaku List -->
      <div class="yaku-section">
        <h3 class="section-title">Yaku</h3>
        <div class="yaku-list">
          {#each result.yaku as yaku}
            <div class="yaku-item">
              <span class="yaku-name">{yaku.name}</span>
              <span class="han-badge" class:yakuman={yaku.is_yakuman}>
                {#if yaku.is_yakuman}
                  役満
                {:else}
                  {yaku.han} han
                {/if}
              </span>
            </div>
          {/each}
        </div>
      </div>

      <!-- Dora -->
      {#if result.dora.total > 0}
        <div class="dora-section">
          <h3 class="section-title">Dora</h3>
          <div class="dora-breakdown">
            {#if result.dora.regular > 0}
              <div class="dora-item">
                <span>Dora</span>
                <span class="han-badge">{result.dora.regular}</span>
              </div>
            {/if}
            {#if result.dora.ura > 0}
              <div class="dora-item">
                <span>Ura Dora</span>
                <span class="han-badge">{result.dora.ura}</span>
              </div>
            {/if}
            {#if result.dora.aka > 0}
              <div class="dora-item">
                <span>Aka Dora</span>
                <span class="han-badge red">{result.dora.aka}</span>
              </div>
            {/if}
          </div>
        </div>
      {/if}

      <!-- Fu Breakdown -->
      <details class="fu-details">
        <summary>Fu Breakdown</summary>
        <div class="fu-breakdown">
          <div class="fu-item">
            <span>Base</span>
            <span>{result.fu_breakdown.base}</span>
          </div>
          {#if result.fu_breakdown.menzen_ron > 0}
            <div class="fu-item">
              <span>Menzen Ron</span>
              <span>+{result.fu_breakdown.menzen_ron}</span>
            </div>
          {/if}
          {#if result.fu_breakdown.tsumo > 0}
            <div class="fu-item">
              <span>Tsumo</span>
              <span>+{result.fu_breakdown.tsumo}</span>
            </div>
          {/if}
          {#if result.fu_breakdown.melds > 0}
            <div class="fu-item">
              <span>Melds</span>
              <span>+{result.fu_breakdown.melds}</span>
            </div>
          {/if}
          {#if result.fu_breakdown.pair > 0}
            <div class="fu-item">
              <span>Pair</span>
              <span>+{result.fu_breakdown.pair}</span>
            </div>
          {/if}
          {#if result.fu_breakdown.wait > 0}
            <div class="fu-item">
              <span>Wait</span>
              <span>+{result.fu_breakdown.wait}</span>
            </div>
          {/if}
          <div class="fu-item total">
            <span>Total (rounded)</span>
            <span>{result.fu_breakdown.raw_total} → {result.fu_breakdown.rounded}</span>
          </div>
        </div>
      </details>

      <!-- Hand Structure -->
      <div class="structure-section">
        <span class="structure-label">Structure:</span>
        <span class="structure-value">{result.hand_structure}</span>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <span class="empty-icon">🀄</span>
      <span class="empty-text">Enter a complete hand to calculate score</span>
    </div>
  {/if}
</div>

<style>
  .score-result {
    min-height: 200px;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 2rem;
    color: var(--text-secondary);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid var(--bg-secondary);
    border-top-color: var(--accent);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    background: rgba(239, 68, 68, 0.1);
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 8px;
  }

  .error-icon {
    font-size: 1.5rem;
  }

  .error-message {
    color: #ef4444;
    font-size: 0.875rem;
  }

  .result-content {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
  }

  /* Score Summary */
  .score-summary {
    text-align: center;
    padding: 1.5rem;
    background: linear-gradient(135deg, rgba(233, 69, 96, 0.1), rgba(15, 52, 96, 0.3));
    border-radius: 12px;
  }

  .score-level {
    font-size: 1.25rem;
    font-weight: bold;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 0.75rem;
  }

  .score-level.mangan { color: #4ade80; }
  .score-level.haneman { color: #60a5fa; }
  .score-level.baiman { color: #a78bfa; }
  .score-level.sanbaiman { color: #f472b6; }
  .score-level.yakuman {
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    font-size: 1.5rem;
  }

  .score-numbers {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .han-fu {
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .han-fu .value {
    font-size: 2rem;
    font-weight: bold;
    color: var(--text-primary);
  }

  .han-fu .label {
    font-size: 0.75rem;
    color: var(--text-secondary);
    text-transform: uppercase;
  }

  .divider {
    font-size: 1.5rem;
    color: var(--text-secondary);
  }

  .total-points {
    margin-bottom: 0.5rem;
  }

  .points-value {
    font-size: 2.5rem;
    font-weight: bold;
    color: var(--accent);
  }

  .points-label {
    font-size: 0.875rem;
    color: var(--text-secondary);
    margin-left: 0.25rem;
  }

  .payment-breakdown {
    font-size: 0.875rem;
    color: var(--text-secondary);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
  }

  .dealer-badge {
    padding: 0.125rem 0.375rem;
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    color: var(--bg-primary);
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  /* Sections */
  .section-title {
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--text-secondary);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 0.5rem 0;
  }

  /* Yaku List */
  .yaku-list {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .yaku-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0.75rem;
    background: var(--bg-secondary);
    border-radius: 6px;
  }

  .yaku-name {
    font-size: 0.875rem;
    color: var(--text-primary);
  }

  .han-badge {
    padding: 0.125rem 0.5rem;
    background: var(--accent);
    color: white;
    border-radius: 4px;
    font-size: 0.75rem;
    font-weight: 600;
  }

  .han-badge.yakuman {
    background: linear-gradient(135deg, #ffd700, #ff8c00);
    color: var(--bg-primary);
  }

  .han-badge.red {
    background: #c41e3a;
  }

  /* Dora */
  .dora-breakdown {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .dora-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.375rem 0.75rem;
    background: var(--bg-secondary);
    border-radius: 6px;
    font-size: 0.875rem;
  }

  /* Fu Details */
  .fu-details {
    background: var(--bg-secondary);
    border-radius: 8px;
    padding: 0.75rem;
  }

  .fu-details summary {
    cursor: pointer;
    font-size: 0.875rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .fu-details[open] summary {
    margin-bottom: 0.75rem;
  }

  .fu-breakdown {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .fu-item {
    display: flex;
    justify-content: space-between;
    font-size: 0.8rem;
    color: var(--text-secondary);
    padding: 0.25rem 0;
  }

  .fu-item.total {
    border-top: 1px solid rgba(255, 255, 255, 0.1);
    padding-top: 0.5rem;
    margin-top: 0.25rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  /* Structure */
  .structure-section {
    font-size: 0.75rem;
    color: var(--text-secondary);
    padding: 0.5rem;
    background: var(--bg-secondary);
    border-radius: 4px;
  }

  .structure-label {
    font-weight: 600;
  }

  .structure-value {
    font-family: 'JetBrains Mono', 'Fira Code', monospace;
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 1rem;
    padding: 3rem;
    color: var(--text-secondary);
  }

  .empty-icon {
    font-size: 3rem;
    opacity: 0.5;
  }

  .empty-text {
    font-size: 0.875rem;
    text-align: center;
  }
</style>
