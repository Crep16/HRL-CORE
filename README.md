# HRL-CORE

Directorio de clientes de HRL (Tauri + Vue 3 + TypeScript).

## Comandos

- `npm install` — instala dependencias.
- `npm run dev` — modo desarrollo en el navegador.
- `npm run build` — revisa tipos y compila.
- `npm run tauri dev` — abre la app de escritorio.

## Estructura

```text
HRL-CORE/
├── index.html            Página de entrada
├── src/
│   ├── main.ts           Arranque de la app
│   ├── App.vue           Une las piezas y guarda el estado general
│   ├── components/       Piezas de la pantalla
│   │   ├── AppTopbar.vue         Barra superior con logo
│   │   ├── ClientDirectory.vue   Lista, búsqueda y filtro de clientes
│   │   ├── ClientProfile.vue     Ficha del cliente seleccionado
│   │   └── NewClientDialog.vue   Formulario "Nuevo cliente"
│   ├── data/clients.ts   Clientes de ejemplo
│   ├── types/client.ts   Tipos de datos (Client, Activity...)
│   ├── utils/text.ts     Funciones auxiliares
│   └── assets/
│       ├── images/       Imágenes (logo)
│       └── styles/       app.css (estilos por secciones) y tailwind.css
└── src-tauri/            Parte de escritorio (Rust)
```

## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
