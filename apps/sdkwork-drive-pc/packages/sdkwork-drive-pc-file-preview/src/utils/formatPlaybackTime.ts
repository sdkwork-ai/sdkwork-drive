/**
 * 媒体播放时间读数：一小时以内 `mm:ss`，超过一小时 `h:mm:ss`。
 *
 * 视频与音频工具栏都要显示「当前 / 总时长」，用的是同一条规则；放在这里而不是各写
 * 一份，是为了让两处读数在边界情况（NaN / 负值 / 超长片）下表现一致。
 */
export function formatPlaybackTime(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) {
    return '00:00';
  }
  const total = Math.floor(seconds);
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const remainder = total % 60;
  const pad = (value: number) => String(value).padStart(2, '0');
  return hours > 0
    ? `${hours}:${pad(minutes)}:${pad(remainder)}`
    : `${pad(minutes)}:${pad(remainder)}`;
}
