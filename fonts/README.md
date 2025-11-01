# Fuentes 8-Bit para NotNative

Esta carpeta contiene las fuentes retro/pixeladas que usa NotNative en el **Modo 8BIT**.

## Fuentes Incluidas

- **VT323** - Fuente de terminal retro estilo VT220
  - Licencia: SIL Open Font License 1.1
  - Autor: Peter Hull (Google Fonts)
  - Uso: Editor de texto en modo 8BIT

## Instalación

### Automática (Recomendada)

Ejecuta el script de instalación incluido:

```bash
./install-fonts.sh
```

Este script:
- Instala las fuentes en `~/.local/share/fonts/notnative`
- Actualiza el caché de fuentes del sistema
- Funciona sin permisos de root

### Manual

#### En Arch Linux / Manjaro

```bash
sudo mkdir -p /usr/share/fonts/truetype/notnative
sudo cp fonts/*.ttf /usr/share/fonts/truetype/notnative/
sudo fc-cache -fv
```

#### En Ubuntu / Debian

```bash
sudo mkdir -p /usr/share/fonts/truetype/notnative
sudo cp fonts/*.ttf /usr/share/fonts/truetype/notnative/
sudo fc-cache -f
```

#### Instalación de Usuario (sin root)

```bash
mkdir -p ~/.local/share/fonts/notnative
cp fonts/*.ttf ~/.local/share/fonts/notnative/
fc-cache -f ~/.local/share/fonts/notnative
```

## Uso

1. Instala las fuentes con el script o manualmente
2. Abre NotNative
3. Haz clic en el botón **8BIT** en el footer
4. ¡Disfruta del modo retro!

## Verificación

Para verificar que las fuentes están instaladas:

```bash
fc-list | grep VT323
```

Deberías ver algo como:
```
/usr/share/fonts/truetype/notnative/VT323-Regular.ttf: VT323:style=Regular
```

## Desinstalación

Para eliminar las fuentes:

```bash
# Si se instalaron como usuario
rm -rf ~/.local/share/fonts/notnative
fc-cache -f

# Si se instalaron en el sistema
sudo rm -rf /usr/share/fonts/truetype/notnative
sudo fc-cache -fv
```

## Licencia

Las fuentes incluidas están bajo SIL Open Font License 1.1.
Ver: https://scripts.sil.org/OFL
