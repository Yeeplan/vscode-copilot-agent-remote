const STORAGE_KEY = 'vscode-remote:macs'

function normalizeAddress(address) {
  if (!address) return ''
  let value = address.trim()
  if (!/^https?:\/\//i.test(value)) {
    value = 'http://' + value
  }
  return value.replace(/\/+$/, '')
}

function loadMacs() {
  try {
    const value = localStorage.getItem(STORAGE_KEY)
    if (value) {
      const parsed = JSON.parse(value)
      if (Array.isArray(parsed) && parsed.length > 0) return parsed
    }
  } catch {
    // ignore
  }

  return [
    {
      id: 'local',
      name: '本机',
      address: 'http://127.0.0.1:3030',
    },
  ]
}

function saveMacs(macs) {
  try {
    const normalized = macs.map((item) => ({
      id: item.id || String(Date.now()) + Math.random().toString(16).slice(2),
      name: item.name.trim(),
      address: normalizeAddress(item.address),
    }))
    localStorage.setItem(STORAGE_KEY, JSON.stringify(normalized))
    return normalized
  } catch {
    return macs
  }
}

function getMacById(id) {
  const list = loadMacs()
  return list.find((item) => item.id === id)
}

export { loadMacs, saveMacs, getMacById, normalizeAddress }
