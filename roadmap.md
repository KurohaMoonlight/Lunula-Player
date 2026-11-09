# Roadmap Arsitektur Audio Engine, Real-Time Safety & Modular DSP Studio

Dokumen ini merangkum peta jalan (*roadmap*) rekayasa teknis Lunula Audio Player yang berpedoman mutlak pada [`principle.md`](principle.md): **"Lunula is honest audio player, no gimmick allowed and ultra-lightweight"**.

---

## 📌 Ringkasan Status Keseluruhan

Seluruh tahapan rekayasa teknis utama pada roadmap telah **selesai 100%** dan terverifikasi melalui continuous integration test suite:

| Tahap | Fokus Utama | Status | Hasil Verifikasi |
| :--- | :--- | :---: | :--- |
| **Tahap 0** | Hot-Unplug & Driver Fallback Resilience | **SELESAI (LULUS)** | Penanganan diskoneksi kabel fisik, debouncing 1.5s, isolasi stream crash, zero-freeze. |
| **Tahap 1** | Real-Time Audio Callback Safety & Bounded Buffers | **SELESAI (LULUS)** | Zero-allocation callback, lock-free $O(1)$ plan swap, GC thread memory destruction, bounded buffer. |
| **Tahap 2** | Formal Graph IR & DAG Validation | **SELESAI (LULUS)** | Declarative `GraphIr`, Kahn's cycle detection, reachability BFS, isolasi node mati. |
| **Tahap 3** | Advanced Modular DSP Suite & Matrix Utilities | **SELESAI (LULUS)** | Time Alignment delay mikrodetik ($180^\circ$ phase), Stereo Matrix (M/S lossless, Bauer crossfeed, swap), Linkwitz-Riley LR4 flat summation. |

---

## 🎯 Jaminan Kualitas & Integritas Audio
1. **154/154 Test Lolos 100%** (`cargo test` unit & integration).
2. **Zero-regression** pada mode Bit-Perfect WASAPI Exclusive, Rubato sinc resampler, dan gapless transition.
3. **Ukuran footprint memori tetap ultra-lightweight** (< 80 MB RAM stabil tanpa beban heap tak terduga).
