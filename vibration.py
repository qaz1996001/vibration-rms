import numpy as np

class Vibration:

    @classmethod
    def acceleration_rms(cls, data, sampling_rate, freq_lower_bound=10, freq_upper_bound=1000):
        # given data in unit of Gs
        # return acc_rms(Gs), vel_rms(m) and dis_pk-pk(m)
        # using inverse FFT transformation to depict the pk-pk displacement
        # welch' error of dis_pk-pk will go bigger if there are fft peaking at neighber frequencies
        # error of dis_pk-pk from rms's 2*1.5 will be go bigger if there are multiplier frequencies
        g = 9.8
        FFT = np.fft.rfft(data)
        freq_ticks = np.fft.rfftfreq(len(data), 1 / sampling_rate)
        FFT_filtered = FFT.copy()
        FFT_filtered[freq_ticks < freq_lower_bound] = 0
        FFT_filtered[freq_upper_bound < freq_ticks] = 0
        freq_ticks[0] = 1.  # 避免分母為零
        tmp = (FFT_filtered / (2 * np.pi * freq_ticks) ** 2)
        tmp[0] = 0 + 0j  # 補入被跳過的直流
        tmp = -np.fft.irfft(tmp)
        dis_pkpk = g * (tmp.max() - tmp.min())
        FFT /= len(FFT)
        FFT = np.abs(FFT)
        FFT_high_pass = FFT[(freq_lower_bound <= freq_ticks)]  # high pass
        acc_rms = np.sqrt(np.sum(FFT_high_pass ** 2)) * 2 ** -0.5  # in unit of G(9.8m/s/s)
        fft_filtered = FFT[(freq_lower_bound <= freq_ticks) & (freq_ticks <= freq_upper_bound)]  # both low and high pass
        freq_ticks_filtered = freq_ticks[
            (freq_lower_bound <= freq_ticks) & (freq_ticks <= freq_upper_bound)]  # both low and high pass
        vel_rms = g * np.sqrt(np.sum(fft_filtered ** 2 / (freq_ticks_filtered * 2 * np.pi) ** 2)) * 2 ** -0.5
        return acc_rms, vel_rms, dis_pkpk

    @classmethod
    def acceleration_rms2(cls, data, sampling_rate, freq_lower_bound=10, freq_upper_bound=1000):
        # using FFT
        # given data in unit of Gs
        # return acc_rms(Gs), vel_rms(m) and dis_rms(m)
        signal_fft = np.fft.rfft(data)
        freq_ticks = np.fft.rfftfreq(len(data), 1 / sampling_rate)
        FFT_abs = np.abs(signal_fft) / len(signal_fft)
        psd_df = FFT_abs ** 2 / 2
        acc_rms = np.sqrt(np.sum(psd_df[(freq_lower_bound <= freq_ticks)]))  # in unit of G(9.8m/s/s)
        g = 9.81
        psd_df = psd_df * g ** 2  # in unit of SI
        psd_df = psd_df[(freq_lower_bound <= freq_ticks) & (freq_ticks <= freq_upper_bound)]
        freq_ticks = freq_ticks[(freq_lower_bound <= freq_ticks) & (freq_ticks <= freq_upper_bound)]
        vel_rms = np.sqrt(np.sum(psd_df / (freq_ticks * 2 * np.pi) ** 2))
        dis_rms = np.sqrt(np.sum(psd_df / (freq_ticks * 2 * np.pi) ** 4))
        dis_pkpk = dis_rms * 2 * np.sqrt(2)
        return acc_rms, vel_rms, dis_pkpk

    @classmethod
    def get_fft_abs_data(cls, np_data, sampling_rate):
        fft_data_temp = np.fft.rfft(np_data)
        fft_abs_data = np.abs(fft_data_temp)
        fft_abs_data = fft_abs_data / (fft_abs_data.shape[0])
        angle_data = np.angle(fft_data_temp)
        freq_ticks = np.fft.rfftfreq(len(np_data), d=1.0 / sampling_rate)
        return fft_abs_data, freq_ticks, angle_data