module.exports = {
  apps: [
    {
      name: 'klip',
      script: './packages/prep/klip',
      args: ['serve'],
      autorestart: true,
      max_restart: 5,
      instances: 1,
      log_date_format: 'YYYY-MM-DD HH:mm:ss',
    },
  ],
};
