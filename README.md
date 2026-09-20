# 🦁 Lions POS — High-Performance Point of Sales System

[![Rust](https://img.shields.io/badge/Rust-2024%20edition-DEA584?style=flat&logo=rust)](https://www.rust-lang.org/)
[![Axum](https://img.shields.io/badge/Axum-0.8-blue?style=flat)](https://github.com/tokio-rs/axum)
[![SQLite](https://img.shields.io/badge/SQLite-WAL%20Mode-003B57?style=flat&logo=sqlite)](https://sqlite.org/)
[![Vue.js](https://img.shields.io/badge/Vue.js-3.x-42b883?style=flat&logo=vuedotjs)](https://vuejs.org/)
[![Vite](https://img.shields.io/badge/Vite-8.x-646cff?style=flat&logo=vite)](https://vitejs.dev/)
[![Tailwind CSS](https://img.shields.io/badge/Tailwind-3.x-38bdf8?style=flat&logo=tailwindcss)](https://tailwindcss.com/)

**Lions POS** adalah sistem *Point of Sales* modern dan berkinerja tinggi yang dibangun menggunakan **Rust (Axum + SQLx + SQLite + JWT)** pada backend dan **Vue 3 (Composition API + Tailwind CSS + Pinia)** pada frontend.

---

## ⚡ Fitur Utama (Phase 1 MVP)

- **Kasir / POS Checkout**:
  - Transaksi penjualan kasir dalam *Atomic Database Transaction*.
  - Dukungan metode pembayaran Tunai (`CASH`), Transfer (`TRANSFER`), `QRIS`, dan **Split Payment**.
  - Validasi stok produk otomatis dan pencatatan mutasi stok real-time.
  - Perhitungan diskon item, diskon manual, dan voucher promosi.
  - Cetak struk kasir & ringkasan transaksi.
- **Manajemen Shift Kasir**:
  - Buka shift kasir dengan modal awal (*starting cash*).
  - Tutup shift kasir dengan kalkulasi otomatis selisih kas fisik vs sistem.
- **Katalog & Inventory**:
  - Manajemen produk, kategori, SKU, barcode, dan foto produk.
  - Pelacakan stok multi-cabang & riwayat mutasi stok.
- **Autentikasi & Hak Akses (RBAC)**:
  - Otentikasi berbasis JWT (Access Token & Refresh Token).
  - Kontrol akses berbasis peran (*Roles & Permissions*).
- **Developer CLI Runner (`lion`)**:
  - Menjalankan backend Rust dan frontend Vue secara bersamaan dalam 1 terminal dengan output log berwarna yang rapi.

---

## 🚀 Memulai Cepat (Quick Start)

### 1. Prasyarat
- [Rust](https://rustup.rs/) (versi terbaru)
- [Node.js](https://nodejs.org/) (v18+) & [pnpm](https://pnpm.io/) (atau `npm`)

### 2. Instalasi Frontend Dependencies
```bash
cd frontend
pnpm install
cd ..
```

### 3. Menjalankan Server Development dengan `lion` CLI
Gunakan binary CLI `lion` untuk menjalankan Rust backend dan Vue frontend secara bersamaan:

```bash
# Menjalankan Backend + Frontend sekaligus
cargo run --bin lion -- dev

# Atau jika ingin menjalankan secara terpisah:
cargo run --bin lion -- backend    # Hanya backend Rust (Port 8090)
cargo run --bin lion -- frontend   # Hanya frontend Vue (Port 5173)
```

> **Opsional**: Anda bisa meng-install CLI `lion` secara global ke Cargo bin path dengan:
> ```bash
> cargo install --path . --bin lion
> # Setelah itu bisa langsung menjalankan:
> lion dev
> ```

---

## 🔑 Akun Login Default

Database SQLite (`lions_pos.db`) dibuat dan di-seed secara otomatis pada startup pertama kali:

| Field | Nilai Default |
|---|---|
| **URL Web** | `http://localhost:5173` |
| **URL API** | `http://127.0.0.1:8090` |
| **Username** | `admin` |
| **Password** | `password123` |
| **Role** | `ADMIN` (Akses Penuh) |

---

## 🏗️ Struktur Proyek

```text
Lions-Pos/
├── Cargo.toml                  # Konfigurasi workspace Rust & dependencies
├── src/
│   ├── main.rs                 # Server entrypoint (lions-pos)
│   ├── lib.rs                  # Rust library root
│   ├── bin/
│   │   └── lion.rs             # CLI Runner (lion dev, lion test, dll.)
│   ├── config/                 # Konfigurasi aplikasi & environment
│   ├── database/               # Koneksi database SQLx & seeder otomatis
│   ├── errors/                 # Unified error response handler
│   ├── handlers/               # Handler HTTP endpoint Axum
│   ├── middleware/             # Middleware JWT Authentication
│   ├── models/                 # Struct entity, request, & response DTOs
│   ├── routes/                 # Routing API & CORS
│   └── state/                  # Shared application state
├── tests/
│   └── pos_integration_test.rs # Pengujian integrasi otomatis
├── frontend/                   # Frontend Vue 3 + Vite
│   ├── src/pages/              # Halaman Kasir, Produk, Cabang, Shift, dll.
│   └── vite.config.js          # Proxy Vite ke backend port 8090
└── src-trash/                  # Arsip kode Java Spring Boot lama (referensi)
```

---

## 🧪 Menjalankan Pengujian (Testing)

Jalankan integration test suite dengan perintah:
```bash
cargo test
# atau via CLI lion
cargo run --bin lion -- test
```

---

## 📄 Lisensi
Didistribusikan di bawah lisensi MIT. Lihat `LICENSE` untuk informasi lebih lanjut.
