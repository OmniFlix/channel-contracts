import { sleep } from './sleep'

export const getFutureTimestamp = (seconds: number): number => {
  const now = new Date()
  const future = new Date(now.getTime() + seconds * 1000)
  return msToNano(future.getTime())
}

export const waitUntil = async (datetime: Date): Promise<void> => {
  const diffMs = datetime.getTime() - new Date().getTime()
  if (diffMs > 0) {
    await sleep(diffMs)
  }
}

export const msToNano = (ms: number): number => {
  return ms * 1000000
}

export const nanoToMs = (nano: number): number => {
  return nano / 1000000
}

export const nanoToDate = (nano: number) => {
  return new Date(nanoToMs(nano));
}

export const nanoToReadableDate = (nano: number) => {
  return nanoToDate(nano).toUTCString();
}