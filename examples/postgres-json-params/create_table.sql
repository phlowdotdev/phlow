-- Script para criar a tabela de usuários
-- Execute este script no seu banco PostgreSQL antes de rodar o exemplo

CREATE TABLE IF NOT EXISTS users (
    id INTEGER PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Inserir alguns dados de exemplo (opcional)
INSERT INTO users (id, name, email) VALUES 
(1, 'João Silva', 'joao@email.com'),
(2, 'Maria Santos', 'maria@email.com'),
(3, 'Pedro Oliveira', 'pedro@email.com')
ON CONFLICT (id) DO NOTHING;

-- Verificar se a tabela foi criada
SELECT * FROM users;
