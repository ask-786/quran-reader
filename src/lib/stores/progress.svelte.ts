class ProgressStore {
  fraction = $state(0);
  juz = $state<number | null>(null);
  hizb = $state<number | null>(null);
  scopeText = $state('this Surah');

  update(fraction: number, juz: number | null, hizb: number | null) {
    this.fraction = Math.min(1, Math.max(0, fraction));
    this.juz = juz;
    this.hizb = hizb;
  }

  reset(scopeText = 'this Surah') {
    this.fraction = 0;
    this.juz = null;
    this.hizb = null;
    this.scopeText = scopeText;
  }
}

export const progressStore = new ProgressStore();
