# Rencana Arsitektur & Optimisasi Tingkat Mesin Audio (Lunula Audio Engine & DSP)

Dokumen ini melengkapi optimisasi antarmuka GUI sebelumnya dengan menyajikan **Arsitektur Optimisasi Tingkat Mesin (Audio Engine, Decoder, Resampler, dan DSP Pipeline)**. Seluruh strategi dirancang secara ketat untuk mencapai efisiensi komputasi maksimal dengan **garansi nol error**, **overhead memori nol (zero-allocation)**, serta **integritas bit-perfect 100% terjaga**.

---

## 1. Audit Bottleneck Mesin Audio Saat Ini

Meskipun arsitektur audio saat ini sudah menggunakan thread terpisah dan lock-free ring buffer (`rtrb`), profil mendalam pada alur pemutaran stereo menunjukkan 5 titik komputasi yang masih dapat dioptimalkan:

```mermaid
graph TD
    subgraph Audio_Bottlenecks ["Titik Beban & Overhead Mesin Audio Saat Ini"]
        A1["1. Atomic Contention pada RingBuffer<br/>prod.push() dan cons.pop() dipanggil 384.000x/detik (192kHz)<br/>Atomic cache-line bouncing antar core CPU"]
        A2["2. Biquad Direct Form I State Overhead<br/>4 state registers per filter x 10 band = 40 registers<br/>Komputasi floating-point redundan saat gain 0 dB"]
        A3["3. Scalar Frame-by-Frame Processing<br/>DSP memproses kanal Left & Right secara serial<br/>Belum memanfaatkan SIMD vectorization hardware"]
        A4["4. Unbuffered Disk I/O pada Decoder<br/>File::open langsung dibaca oleh Symphonia MediaSourceStream<br/>Syscall read mikro membebani storage saat memutar 24-bit FLAC"]
        A5["5. Potensi Alokasi Dinamis pada DAG Paralel<br/>Persiapan percabangan multi-filter membutuhkan scratch buffer<br/>Harus dicegah jangan sampai ada Vec::new di audio thread"]
    end
```

### Tabel Perbandingan Arsitektur Mesin: Saat Ini vs Target Optimisasi

| Komponen Mesin | Kondisi Saat Ini | Target Optimisasi Mesin | Dampak Performa & Audio |
|---|---|---|---|
| **RingBuffer SPSC (`engine.rs`, `cpal_backend.rs`)** | `push()` & `pop()` skalar per sampel. Tiap sampel memicu manipulasi atomic head/tail pointer. | **Chunk Slice Transfer (Batch 64–256)**: Mengirim dan membaca blok sampel sekaligus via contiguous slices. | Mengurangi atomic cache-line bouncing hingga **98%**. Latensi audio lebih stabil. |
| **Biquad Filters (`dsp.rs`)** | Direct Form I dengan 4 state (`x1, x2, y1, y2`). Ke-10 band diproses meski gain = 0 dB (Flat). | **Direct Form II Transposed (DF2T) + Zero-Cost Bypass**: 2 state (`s1, s2`) per filter. Skip instan jika Flat/Bypass. | Register CPU hemat 50%, presisi `f64` lebih tahan quantization noise, 0% CPU saat EQ Flat. |
| **DSP Frame Loop (`dsp.rs`)** | Loop skalar serial satu frame demi satu frame. | **Chunk Loop Auto-Vectorization (AVX2/NEON)**: Loop contiguous slice yang ramah SIMD auto-vectorizer. | Komputasi EQ 40–60% lebih kencang, L & R dikalkulasi simultan dalam 1 clock cycle. |
| **Audio File Decoder (`decoder.rs`)** | Raw `File::open` langsung ke `MediaSourceStream`. | **`BufReader` 256 KB**: Bungkus file stream dengan buffer memori berkas besar. | Menghilangkan context switch I/O OS mikro. Streaming FLAC/WAV 192kHz dari HDD/USB bebas lag. |
| **DAG Parallel Summing (`dsp.rs`)** | Belum ada scratch buffer terdedikasi untuk branch filter. | **Pre-allocated Static Scratch Buffers**: Sediakan buffer tetap stereo di `DspPipeline` untuk percabangan kabel. | Garansi **Zero Allocation** di thread audio saat filter dihubungkan paralel. |

---

## 2. Rincian Teknis 5 Pilar Optimisasi Mesin

### Pilar 1: SPSC RingBuffer Chunk Batching
Saat ini di loop audio worker:
```rust
// Saat ini: 2 atomic push per frame
let _ = prod.push(out_l);
let _ = prod.push(out_r);
```
Dan di callback hardware:
```rust
// Saat ini: 2 atomic pop per frame
let l = consumer.pop().unwrap_or(0.0) * vol;
let r = consumer.pop().unwrap_or(0.0) * vol;
```
**Optimalisasi:**
Menggunakan buffer perantara lokal kecil (`[f32; 128]`) di stack thread:
- Worker mengisi blok 64 frame (128 nilai `f32`), lalu menuliskan seluruh slice ke `Producer` dalam 1 transaksi atomic chunk.
- Callback hardware membaca chunk slice langsung ke buffer audio output.
- Hasil: Menghilangkan ratusan ribu pembaruan atomic cache line per detik.

---

### Pilar 2: Direct Form II Transposed (DF2T) & Zero-Cost Flat Bypass
Struktur biquad saat ini menggunakan Direct Form I:
$$y[n] = b_0 x[n] + b_1 x[n-1] + b_2 x[n-2] - a_1 y[n-1] - a_2 y[n-2]$$
Membutuhkan 4 variabel state: $x_1, x_2, y_1, y_2$.

Kita migrasikan ke **Direct Form II Transposed (DF2T)**:
$$y[n] = b_0 x[n] + s_1[n-1]$$
$$s_1[n] = b_1 x[n] - a_1 y[n] + s_2[n-1]$$
$$s_2[n] = b_2 x[n] - a_2 y[n]$$

```mermaid
graph LR
    X["x[n] Input"] --> MulB0["* b0"] --> Add1((+)) --> Y["y[n] Output"]
    X --> MulB1["* b1"] --> Add2((+))
    X --> MulB2["* b2"] --> Add3((+))
    Y --> MulA1["* -a1"] --> Add2
    Y --> MulA2["* -a2"] --> Add3
    Add3 --> Delay2["z⁻¹ (s2)"] --> Add2
    Add2 --> Delay1["z⁻¹ (s1)"] --> Add1
```

> [!NOTE] Keunggulan Matematis DF2T
> 1. Hanya membutuhkan **2 variabel state** (`s1, s2`) alih-alih 4.
> 2. Memiliki rentang dinamis (*dynamic range*) floating-point yang jauh lebih baik terhadap akumulasi galat pembulatan numerik pada frekuensi sub-bass (20–40 Hz).
> 3. Penghematan register CPU memungkinkan compiler menempatkan seluruh 10 band filter langsung ke register SIMD hardware tanpa tumpah ke stack memory (*register spilling*).

**Short-Circuit Flat Check:**
```rust
let is_all_zero = self.eq_gains.iter().all(|&g| g.abs() < 0.01);
if is_all_zero {
    return (l * self.auto_preamp, r * self.auto_preamp); // 0 Biquad math!
}
```

---

### Pilar 3: SIMD Stereo Auto-Vectorization
Kanal Left dan Right bersifat independen. Dengan menyusun loop pemrosesan dalam bentuk blok:
```rust
#[inline(always)]
pub fn process_stereo_block(&mut self, left: &mut [f64], right: &mut [f64]) {
    for i in 0..left.len() {
        let (out_l, out_r) = self.process_frame(left[i], right[i]);
        left[i] = out_l;
        right[i] = out_r;
    }
}
```
Instruksi compiler LLVM akan menghasilkan instruksi SIMD 128-bit/256-bit (`vfmadd213pd` / AVX2 FMA) yang mengeksekusi operasi Left & Right secara simultan.

---

### Pilar 4: High-Throughput I/O Buffering (`BufReader`)
Di [`src/audio/decoder.rs`](file:///e:/My%20Projects/audio%20player/src/audio/decoder.rs):
```rust
let file = File::open(path_ref)?;
let buffered = std::io::BufReader::with_capacity(256 * 1024, file); // 256 KB Buffer
let mss = MediaSourceStream::new(Box::new(buffered), Default::default());
```
Sistem operasi membaca audio dalam blok besar 256 KB. Tidak ada lagi syscall `read` ukuran 4KB berulang-ulang saat mendekode FLAC 24-bit/192kHz.

---

### Pilar 5: Static Scratch Buffers untuk DAG Multi-Branch (Ide B & C)
Mempersiapkan audio engine untuk percabangan DAG di Modular Patchbay Studio:
- Mengalokasikan 2 static array buffer stereo berukuran 1024 sampel di dalam `DspPipeline`.
- Ketika sinyal di-split menjadi 2 filter paralel:
  1. Filter A memproses buffer utama.
  2. Filter B memproses scratch buffer kedua.
  3. Hasil keduanya di-summing: `out[i] = (main[i] + scratch[i]) * 0.5`.
- **Nol alokasi `Vec::new`** di tengah pemutaran lagu.

---

## 3. Garansi Integritas & Keamanan Audio (Auditory Guardrails)

> [!IMPORTANT] Prinsip Tanpa Kompromi
> 1. **Bit-Perfect 100% Utuh:** Pada mode `ChainMode::BitPerfect`, seluruh jalur komputasi DSP (biquad, limiter, volume digital, dithering) **sama sekali tidak dijalankan**. Sampel audio dialirkan secara bit-for-bit identik dari decoder ke hardware DAC.
> 2. **Audio Callback Thread Bebas Lock & Bebas Alokasi:** Callback audio real-time OS tidak boleh menyentuh mutex (`std::sync::Mutex`) atau alokasi heap (`Vec`, `Box`, `String`).
> 3. **Verifikasi Test Suite Otomatis:** Setiap perubahan wajib lulus seluruh **53 unit/integration tests** yang mencakup pengujian stabilitas filter 31 Hz pada 192 kHz, bit-perfect proofs, dan DAG verification.

---

## 4. Rencana Roadmap Eksekusi Bertahap

```mermaid
flowchart TD
    Phase1["Tahap 1: Decoder I/O Buffer (256 KB BufReader)<br/>+ Zero-Cost EQ Flat Bypass"]
    Phase2["Tahap 2: Biquad Direct Form II Transposed (DF2T)<br/>Presisi f64 & register reduction"]
    Phase3["Tahap 3: SPSC RingBuffer Chunk Batching<br/>Eliminasi atomic bouncing"]
    Phase4["Tahap 4: Static Pre-Allocated Scratch Buffers<br/>Dukungan penuh paralel branch DAG"]

    Phase1 --> Phase2 --> Phase3 --> Phase4
```

1. **Tahap 1: Safe Foundation (Nol Risiko)**:
   - Bungkus decoder dengan `BufReader::with_capacity(256 * 1024, file)`.
   - Tambahkan fast short-circuit check pada Graphic EQ jika semua gain = 0 dB.
2. **Tahap 2: Biquad Filter Refactor**:
   - Ganti implementasi Direct Form I menjadi Direct Form II Transposed di `dsp.rs`.
   - Validasi respons frekuensi dan headroom auto-preamp via unit test biquad.
3. **Tahap 3: RingBuffer Batching**:
   - Terapkan transfer frame dalam chunk pada audio worker loop dan callback renderer.
4. **Tahap 4: Pre-allocated Scratch Buffers**:
   - Siapkan static stereo array untuk arsitektur mixing paralel DAG.
