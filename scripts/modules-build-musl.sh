#!/bin/bash

set -e

# Diretórios
DEST_DIR="./phlow_packages"
MODULES_DIR="./modules"
TARGET="x86_64-unknown-linux-musl"
BUILD_DIR="target/$TARGET/release"

# Cria destino
mkdir -p "$DEST_DIR"

# Build com cross para cada módulo
for module in "$MODULES_DIR"/*; do
    if [ -d "$module" ]; then
        module_name=$(basename "$module")

        echo "🔧 Buildando módulo: $module_name"
        cross build --release --target $TARGET --manifest-path "$MODULES_DIR/$module_name/Cargo.toml"

        lib_file="$BUILD_DIR/lib$module_name.a"
        wrapper_file="$module/wrapper.c"
        output_dir="$DEST_DIR/$module_name"
        output_so="$output_dir/module.so"

        mkdir -p "$output_dir"

        if [ ! -f "$wrapper_file" ]; then
            echo "⚠️  wrapper.c não encontrado para $module_name. Criando wrapper padrão..."
            cat > "$wrapper_file" <<EOF
extern void plugin();

void plugin_entry() {
    plugin();
}
EOF
        fi

        echo "📦 Gerando .so com musl-gcc"
        musl-gcc -shared -o "$output_so" "$wrapper_file" "$lib_file" -Wl,--whole-archive -Wl,--no-whole-archive

        echo "✅ .so gerado: $output_so"

        # Copia o phlow.yaml se existir
        yaml_file="$module/phlow.yaml"
        if [ -f "$yaml_file" ]; then
            cp "$yaml_file" "$output_dir/phlow.yaml"
            echo "📄 Copiado: $yaml_file"
        else
            echo "⚠️  phlow.yaml não encontrado em $module"
        fi
    fi

done
