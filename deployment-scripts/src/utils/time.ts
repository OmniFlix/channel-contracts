import { logger } from './logger'

export const generateNowInNano = async () => {
    let now = new Date()
    let nowNano = now.getTime() * 1000000
    return nowNano
}

export const sleepUntil = async (date: Date) => {
    let now = new Date()
    let sleepTime = (date.getTime() - now.getTime()) + 10000
    logger.log(1, `Sleeping for ${sleepTime / 1000} seconds`)
    if (sleepTime > 0) {
        await new Promise((resolve) => setTimeout(resolve, sleepTime))
    }
}

export const generateFeatureTimestampInNano = async (secs: Number) => {
    let now = new Date()
    let nowNano = now.getTime() * 1000000
    let futureNano = nowNano + Number(secs) * 1000000000
    return futureNano
}
