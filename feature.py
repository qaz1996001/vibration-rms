import numpy as np
import scipy


def both_Fea(signal_, PL, axis, f, create_time):
    """
    提取時域和頻域特徵
    參考文獻: Lei, Y., He, Z., Zi, Y., & Hu, Q. (2007).
    Fault diagnosis of rotating machinery based on multiple ANFIS combination with GAs.
    Mechanical systems and signal processing, 21(5), 2280-2294.
    """
    time_domain_feature = time_domain_fea(signal_=signal_)
    freq_domain_feature = freq_domain_fea(PL=PL, f=f)
    return {
        'time_domain_feature': time_domain_feature,
        'freq_domain_feature': freq_domain_feature,
        'axis': axis,
        'create_time': create_time,
    }


def freq_domain_fea(PL, f):
    """
    提取频域特征 12類
    :param PL: FFT magnitude array
    :param f: frequency array
    :return: frequency domain features (12)
    """
    x = f
    y = PL
    K = len(y)

    f_12 = np.mean(y)

    f_13 = np.var(y)

    f_14 = (np.sum((y - f_12) ** 3)) / (K * ((np.sqrt(f_13)) ** 3))

    f_15 = (np.sum((y - f_12) ** 4)) / (K * ((f_13) ** 2))

    f_16 = (np.sum(x * y)) / (np.sum(y))

    f_17 = np.sqrt((np.mean(((x - f_16) ** 2) * (y))))

    f_18 = np.sqrt((np.sum((x ** 2) * y)) / (np.sum(y)))

    f_19 = np.sqrt((np.sum((x ** 4) * y)) / (np.sum((x ** 2) * y)))

    f_20 = (np.sum((x ** 2) * y)) / (np.sqrt((np.sum(y)) * (np.sum((x ** 4) * y))))

    f_21 = f_17 / f_16

    f_22 = (np.sum(((x - f_16) ** 3) * y)) / (K * (f_17 ** 3))

    f_23 = (np.sum(((x - f_16) ** 4) * y)) / (K * (f_17 ** 4))

    f_fea = np.array([f_12, f_13, f_14, f_15, f_16, f_17, f_18, f_19, f_20, f_21, f_22, f_23])

    return f_fea


def time_domain_fea(signal_):
    """
    提取時域特徵 13 類
    """
    N = len(signal_)
    y = signal_

    t_max = np.max(y)

    t_min = np.min(y)

    # 1_均值（平均幅值）
    t_mean_1 = np.mean(y)

    # 2_标准差
    t_std_2 = np.std(y, ddof=1)

    # 3_方根幅值
    t_fgf_3 = ((np.mean(np.sqrt(np.abs(y))))) ** 2

    # 4_RMS均方根
    t_rms_4 = np.sqrt((np.mean(y ** 2)))

    # 5_峰峰值
    t_pp_5 = 0.5 * (t_max - t_min)

    # 6_偏度 skewness
    t_skew_6 = scipy.stats.skew(y)

    # 7_峭度 Kurtosis
    t_kur_7 = scipy.stats.kurtosis(y)

    # 8_峰值因子 Crest Factor
    t_cres_8 = np.max(np.abs(y)) / t_rms_4

    # 9_裕度因子  Clearance Factor
    t_clear_9 = np.max(np.abs(y)) / t_fgf_3

    # 10_波形因子 Shape Factor
    t_shape_10 = (N * t_rms_4) / (np.sum(np.abs(y)))

    # 11_脉冲指数 Impulse Factor
    t_imp_11 = (np.max(np.abs(y))) / (np.mean(np.abs(y)))

    t_fea = np.array([t_max, t_min, t_mean_1, t_std_2, t_fgf_3, t_rms_4, t_pp_5,
                      t_skew_6, t_kur_7, t_cres_8, t_clear_9, t_shape_10, t_imp_11])

    return t_fea
