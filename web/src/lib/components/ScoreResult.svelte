<script lang="ts">
  import type { ScoringOutput } from '../agari';

  interface Props {
    result: ScoringOutput | null;
    error?: string | null;
    loading?: boolean;
  }

  let { result, error = null, loading = false }: Props = $props();

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
      <span class="error-message">{error}</span>
    </div>
  {:else if result}
    <div class="result-content">
      {#if result.inferred_winning_tile}
        <div class="inferred-warning">
          Winning tile inferred as <strong>{result.inferred_winning_tile}</strong>
        </div>
      {/if}

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
            <span class="unit">han</span>
          </div>
          <span class="divider">/</span>
          <div class="han-fu">
            <span class="value">{result.fu}</span>
            <span class="unit">fu</span>
          </div>
        </div>

        <div class="total-points">
          <span class="points-value">{result.payment.total.toLocaleString()}</span>
          <span class="points-label">pts</span>
        </div>

        <div class="payment-breakdown">
          {formatPayment(result.payment, result.is_dealer, isTsumo)}
          {#if result.is_dealer}
            <span class="dealer-tag">Dealer</span>
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
              <span class="yaku-han" class:yakuman={yaku.is_yakuman}>
                {#if yaku.is_yakuman}役満{:else}{yaku.han}{/if}
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
                <span class="dora-count">{result.dora.regular}</span>
              </div>
            {/if}
            {#if result.dora.ura > 0}
              <div class="dora-item">
                <span>Ura</span>
                <span class="dora-count ura">{result.dora.ura}</span>
              </div>
            {/if}
            {#if result.dora.aka > 0}
              <div class="dora-item">
                <span>Aka</span>
                <span class="dora-count aka">{result.dora.aka}</span>
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
            <span>Total</span>
            <span>{result.fu_breakdown.raw_total} → {result.fu_breakdown.rounded}</span>
          </div>
        </div>
      </details>

      <!-- Hand Structure -->
      <div class="structure-section">
        <span class="structure-label">Structure:</span>
        <code class="structure-value">{result.hand_structure}</code>
      </div>
    </div>
  {:else}
    <div class="empty-state">
      <span class="empty-text">Enter a complete hand to calculate score</span>
    </div>
  {/if}
</div>

<style>
  .score-result {
    min-height: 150px;
  }

  .loading {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    padding: var(--space-6);
    color: var(--text-muted);
  }

  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--border);
    border-top-color: var(--accent);
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }

  .error {
    padding: var(--space-3);
    background: var(--error-muted);
    border: 1px solid var(--error);
  }

  .error-message {
    color: var(--error);
    font-size: 0.8125rem;
  }

  .inferred-warning {
    padding: var(--space-2) var(--space-3);
    background: var(--warning-muted);
    border: 1px solid var(--warning);
    color: var(--warning);
    font-size: 0.75rem;
    margin-bottom: var(--space-3);
  }

  .inferred-warning strong {
    font-family: var(--font-mono);
  }

  .result-content {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
  }

  /* Score Summary */
  .score-summary {
    text-align: center;
    padding: var(--space-4);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
  }

  .score-level {
    font-size: 0.875rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: var(--space-3);
    font-family: var(--font-mono);
  }

  .score-level.mangan {
    color: var(--success);
  }
  .score-level.haneman {
    color: #60a5fa;
  }
  .score-level.baiman {
    color: #a78bfa;
  }
  .score-level.sanbaiman {
    color: #f472b6;
  }
  .score-level.yakuman {
    color: var(--warning);
  }

  .score-numbers {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-3);
    margin-bottom: var(--space-3);
  }

  .han-fu {
    display: flex;
    align-items: baseline;
    gap: var(--space-1);
  }

  .han-fu .value {
    font-size: 1.5rem;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--text-primary);
  }

  .han-fu .unit {
    font-size: 0.6875rem;
    color: var(--text-muted);
    text-transform: uppercase;
  }

  .divider {
    font-size: 1rem;
    color: var(--text-muted);
  }

  .total-points {
    margin-bottom: var(--space-2);
  }

  .points-value {
    font-size: 2rem;
    font-weight: 700;
    font-family: var(--font-mono);
    color: var(--accent);
  }

  .points-label {
    font-size: 0.75rem;
    color: var(--text-muted);
    margin-left: var(--space-1);
  }

  .payment-breakdown {
    font-size: 0.75rem;
    color: var(--text-secondary);
    font-family: var(--font-mono);
    display: flex;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
  }

  .dealer-tag {
    padding: var(--space-1) var(--space-2);
    background: var(--warning-muted);
    border: 1px solid var(--warning);
    color: var(--warning);
    font-size: 0.625rem;
    font-weight: 600;
    text-transform: uppercase;
  }

  /* Sections */
  .section-title {
    font-size: 0.6875rem;
    font-weight: 600;
    color: var(--text-muted);
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin: 0 0 var(--space-2) 0;
  }

  /* Yaku List */
  .yaku-list {
    display: flex;
    flex-direction: column;
    gap: 1px;
    background: var(--border);
    border: 1px solid var(--border);
  }

  .yaku-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
  }

  .yaku-name {
    font-size: 0.8125rem;
    color: var(--text-primary);
  }

  .yaku-han {
    font-size: 0.6875rem;
    font-family: var(--font-mono);
    font-weight: 600;
    padding: var(--space-1) var(--space-2);
    background: var(--accent-muted);
    border: 1px solid var(--accent);
    color: var(--accent);
  }

  .yaku-han.yakuman {
    background: var(--warning-muted);
    border-color: var(--warning);
    color: var(--warning);
  }

  /* Dora */
  .dora-breakdown {
    display: flex;
    flex-wrap: wrap;
    gap: var(--space-2);
  }

  .dora-item {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    font-size: 0.8125rem;
  }

  .dora-count {
    font-family: var(--font-mono);
    font-weight: 600;
    color: var(--accent);
  }

  .dora-count.ura {
    color: #a78bfa;
  }
  .dora-count.aka {
    color: var(--man-color);
  }

  /* Fu Details */
  .fu-details {
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    padding: var(--space-3);
  }

  .fu-details summary {
    cursor: pointer;
    font-size: 0.8125rem;
    font-weight: 500;
    color: var(--text-secondary);
  }

  .fu-details[open] summary {
    margin-bottom: var(--space-3);
    padding-bottom: var(--space-2);
    border-bottom: 1px solid var(--border);
  }

  .fu-breakdown {
    display: flex;
    flex-direction: column;
    gap: var(--space-1);
  }

  .fu-item {
    display: flex;
    justify-content: space-between;
    font-size: 0.75rem;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    padding: var(--space-1) 0;
  }

  .fu-item.total {
    border-top: 1px solid var(--border);
    padding-top: var(--space-2);
    margin-top: var(--space-1);
    font-weight: 600;
    color: var(--text-primary);
  }

  /* Structure */
  .structure-section {
    font-size: 0.75rem;
    color: var(--text-muted);
    padding: var(--space-2) var(--space-3);
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    display: flex;
    gap: var(--space-2);
  }

  .structure-label {
    font-weight: 600;
  }

  .structure-value {
    font-family: var(--font-mono);
    color: var(--text-secondary);
  }

  /* Empty State */
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: var(--space-6);
    color: var(--text-muted);
  }

  .empty-text {
    font-size: 0.8125rem;
  }
</style>
