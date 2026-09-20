# Panduan Kontribusi Lions POS 🦁

Terima kasih atas ketertarikan Anda untuk berkontribusi pada proyek **Lions POS**!

---

## 🛠️ Alur Kerja Pengembangan

### 1. Prasyarat Lingkungan
Pastikan Anda telah menginstal:
- **Rust toolchain** (versi terbaru, gunakan `rustup update`)
- **Node.js** v18+ & **pnpm** (atau `npm`)

### 2. Menjalankan Mode Development
Gunakan CLI `lion` untuk menjalankan backend dan frontend secara bersamaan:
```bash
cargo run --bin lion -- dev
```
Log dari backend dan frontend akan ditampilkan secara real-time dengan label berwarna `[rust]` dan `[vue]`.

### 3. Standar Penulisan Kode (Code Style)
- **Rust Backend**:
  - Gunakan `cargo fmt` untuk memformat kode secara konsisten.
  - Jalankan `cargo clippy` untuk memastikan tidak ada peringatan kode atau bad practice.
  - Gunakan error handling terpadu melalui enum `AppError` di `src/errors/mod.rs`.
- **Frontend Vue**:
  - Ikuti konvensi Vue 3 Composition API (`<script setup>`).
  - Gunakan utility class Tailwind CSS yang telah dikonfigurasi.

### 4. Menjalankan Pengujian
Sebelum membuat Pull Request atau commit, pastikan semua tes lulus:
```bash
cargo test
# atau
cargo run --bin lion -- test
```

### 5. Membuat Pull Request
1. Fork repository ini dan buat branch fitur baru:
   ```bash
   git checkout -b feature/nama-fitur-baru
   ```
2. Lakukan commit dengan pesan yang deskriptif dan jelas.
3. Push branch Anda dan buat Pull Request ke branch `main`.

---

Jika ada pertanyaan atau diskusi desain sistem, silakan buat issue atau diskusikan di forum komunitas repository!